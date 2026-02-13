use agent_state::{
    Bundle, CbbaGossip, Config, SharedBundle, SharedWinners, Task, TaskContext, TaskId, Winner,
    Winners,
};
use common::time::now;
use common::{RmpSerializable, SerializationError};
use std::cmp::Ordering;
use std::collections::HashMap;
use thiserror::Error;
use tokio::time::{self, Duration};
use tracing::{error, info, warn};
use transport::Transport;
use udp_transport::UdpTransport;

const MAX_GOSSIP_SIZE: usize = 2048;

#[derive(Debug, Error)]
pub enum CbbaError {
    #[error("I/O error")]
    Io(#[from] std::io::Error),
    #[error("Transport error: {0}")]
    Transport(#[from] udp_transport::TransportError),
    #[error("Serialization error: {0}")]
    Serialization(#[from] SerializationError),
}

enum ConflictDecision {
    RemoteWins,
    LocalWins,
}

pub struct CbbaRunner {
    config: Config,
    shared_bundle: SharedBundle,
    shared_winners: SharedWinners,
    tasks: HashMap<TaskId, Task>,
}

impl CbbaRunner {
    pub fn new(
        config: Config,
        shared_bundle: SharedBundle,
        shared_winners: SharedWinners,
        tasks: Vec<Task>,
    ) -> Self {
        let tasks = tasks.into_iter().map(|task| (task.id, task)).collect();
        Self {
            config,
            shared_bundle,
            shared_winners,
            tasks,
        }
    }
}

impl CbbaRunner {
    pub async fn start(&self) -> Result<(), CbbaError> {
        let port = self.config.cbba_port;
        let cbba_timeout = self.config.cbba_timeout;
        let agent_id = self.config.agent_id;

        let mut sender = UdpTransport::new_sender(port).await?;
        let mut receiver = UdpTransport::new_receiver(port).await?;
        let mut bytes = [0u8; MAX_GOSSIP_SIZE];

        let timer = time::sleep(Duration::from_secs(cbba_timeout));
        tokio::pin!(timer);

        info!("⚙️ Starting CBBA process...");

        let (mut bundle, mut winners) = self.init_bundle_and_winners();
        // Send initial gossip
        let gossip = winners.to_gossip(agent_id);
        send_gossip(&mut sender, &gossip).await?;

        loop {
            tokio::select! {
                result = receiver.recv(&mut bytes) => {
                    let size = match result {
                        Ok(size) => size,
                        Err(e) => {
                            error!("Error receiving gossip: {}", e);
                            continue;
                        }
                    };

                    let Ok(gossip) = CbbaGossip::from_bytes(&bytes[..size]) else {
                        warn!("Dropping invalid gossip.");
                        continue;
                    };

                    if process_gossip(agent_id, &mut bundle, &mut winners, &gossip) {
                        self.rebid_and_sort(&mut bundle, &mut winners);
                        let gossip = winners.to_gossip(agent_id);
                        send_gossip(&mut sender, &gossip).await?;
                    }
                },
                _ = &mut timer => {
                    info!("Timeout reached.");
                    break;
                }
            }
        }

        // Update the agent state with the new bundle and winners
        let mut shared_bundle = self.shared_bundle.write().await;
        let mut shared_winners = self.shared_winners.write().await;

        *shared_bundle = bundle;
        *shared_winners = winners;

        info!("Bundle and winners updated.");

        Ok(())
    }

    fn init_bundle_and_winners(&self) -> (Bundle, Winners) {
        let mut bundle = Bundle::new();
        let mut winners = Winners::new();

        bundle.replace(self.tasks.keys().cloned().collect());
        self.rebid_and_sort(&mut bundle, &mut winners);

        (bundle, winners)
    }

    /// Rebids tasks in the bundle
    fn rebid_and_sort(&self, bundle: &mut Bundle, winners: &mut Winners) {
        let agent_id = self.config.agent_id;
        // Step 1
        let mut tasks_with_bids = self.build_tasks_with_bids(bundle);
        // Step 2
        tasks_with_bids.sort_by(|a, b| match (a.1.is_finite(), b.1.is_finite()) {
            (true, true) => {
                if a.1 < b.1 {
                    Ordering::Greater
                } else if a.1 > b.1 {
                    Ordering::Less
                } else {
                    Ordering::Equal
                }
            }
            (false, true) => {
                error!("Task {} has invalid bid", a.0);
                Ordering::Greater
            }
            (true, false) => {
                error!("Task {} has invalid bid", b.0);
                Ordering::Less
            }
            (false, false) => {
                error!("Both Task {} and Task {} have invalid bids", a.0, b.0);
                Ordering::Equal
            }
        });

        // Step 3: clear current bundle and rebuild in sorted order
        bundle.clear();
        for (task_id, bid) in tasks_with_bids {
            bundle.insert(task_id);
            winners.insert(task_id, agent_id, bid, now()); // fresh timestamp
        }
    }

    /// Build a vector of tasks in Bundle with their bids
    fn build_tasks_with_bids(&self, bundle: &Bundle) -> Vec<(TaskId, f64)> {
        let ctx = TaskContext::new(self.tasks.len());
        bundle
            .task_ids()
            .iter()
            .map(|task_id| {
                let task = self.tasks.get(task_id).expect("Failed to fetch task");
                let bid = calculate_task_bid(&ctx, &task);
                (*task_id, bid)
            })
            .collect()
    }
}

/// If bundle or winners changed, return true, false otherwise
fn process_gossip(
    agent_id: u32,
    bundle: &mut Bundle,
    winners: &mut Winners,
    gossip: &CbbaGossip,
) -> bool {
    if agent_id == gossip.agent_id {
        return false;
    }

    let mut bundle_changed = false;

    // Loop through the winners in the received gossip
    for remote in gossip.winners.iter() {
        let task_id = remote.task_id;

        if let Some(local) = winners.get(task_id) {
            if remote.agent_id == local.agent_id {
                continue; // Dude! We've already agreed!
            }

            let winner = match compare(remote, local) {
                ConflictDecision::RemoteWins => remote,
                ConflictDecision::LocalWins => local,
            };

            let was_in_bundle = bundle.contains(task_id);
            if winner.agent_id == agent_id {
                bundle.insert(task_id);
            } else if was_in_bundle {
                bundle.truncate_after(task_id);
            }

            winners.insert(task_id, winner.agent_id, winner.bid, winner.ts);

            let now_in_bundle = bundle.contains(task_id);
            if was_in_bundle != now_in_bundle {
                bundle_changed = true;
            }
        } else {
            info!(
                "Agent does not have a local winner for the task {}. Potential inconsistency in the task store.",
                task_id
            );

            bundle.remove(task_id);
            winners.insert(task_id, remote.agent_id, remote.bid, remote.ts);
            bundle_changed = true;
        }
    }

    bundle_changed
}

fn compare(remote: &Winner, local: &Winner) -> ConflictDecision {
    if remote.bid > local.bid {
        return ConflictDecision::RemoteWins;
    }

    if remote.bid < local.bid {
        return ConflictDecision::LocalWins;
    }

    if remote.ts > local.ts {
        return ConflictDecision::RemoteWins;
    }

    if remote.ts < local.ts {
        return ConflictDecision::LocalWins;
    }

    if remote.agent_id < local.agent_id {
        return ConflictDecision::RemoteWins;
    }

    ConflictDecision::LocalWins
}

fn calculate_task_bid(ctx: &TaskContext, task: &Task) -> f64 {
    let distance = ctx.agent_location.distance_to(&task.location);
    let distance_score = 1.0 + distance;
    let task_penalty = 1.0 + ctx.task_count as f64 * ctx.task_count_weight;
    let priority = task.priority as f64;
    1000.0 * (ctx.energy * priority) / (task_penalty * distance_score)
}

async fn send_gossip(sender: &mut UdpTransport, gossip: &CbbaGossip) -> Result<(), CbbaError> {
    let bytes = gossip.to_bytes()?;
    sender.send(&bytes).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> (Bundle, Winners) {
        let bundle = Bundle::new();
        let winners = Winners::new();

        (bundle, winners)
    }

    // Test that the state has changed because the local agent wins the task.
    // Check that the winner for the task changes to the local agent.
    // Ensure that the task has been added to the local bundle.
    #[test]
    fn local_higher_bid_wins() {
        let local_agent_id = 1000;
        let remote_agent_id = 1001;

        let task_id = 2000;
        let bid = 10.0;
        let lower_bid = bid - 1.0;
        let ts = 10;

        let (mut bundle, mut winners) = setup();

        winners.insert(task_id, local_agent_id, bid, ts);
        // Remote gossip
        let gossip = CbbaGossip {
            agent_id: remote_agent_id,
            winners: vec![Winner {
                task_id,
                agent_id: remote_agent_id, // another agent
                bid: lower_bid,            // Lower bid
                ts,                        // Same timestamp
            }],
        };

        let changed = process_gossip(local_agent_id, &mut bundle, &mut winners, &gossip);
        assert!(changed, "state should be marked as changed");

        let winner = winners.get(task_id).expect("winner missing");
        assert_eq!(winner.agent_id, local_agent_id, "remote agent should win");
        assert!(
            bundle.contains(task_id),
            "task must be added to local bundle after win"
        );
    }

    // Test that the state has changed because the local agent lost the task.
    // Check that the winner for the task changes to the remote agent.
    // Ensure that the task has been removed from the local bundle.
    #[test]
    fn remote_higher_bid_wins() {
        let local_agent_id = 1000;
        let remote_agent_id = 1001;

        let task_id = 2000;
        let bid = 10.0;
        let higher_bid = bid + 5.0;
        let ts = 10; // timestamp

        let (mut bundle, mut winners) = setup();

        bundle.replace(vec![task_id]);
        winners.insert(task_id, local_agent_id, bid, ts);

        // Remote gossip
        let gossip = CbbaGossip {
            agent_id: remote_agent_id,
            winners: vec![Winner {
                task_id,
                agent_id: remote_agent_id, // another agent
                bid: higher_bid,           // Higher bid
                ts,                        // Same timestamp
            }],
        };

        let changed = process_gossip(local_agent_id, &mut bundle, &mut winners, &gossip);
        assert!(changed, "state should be marked as changed");

        let winner = winners.get(task_id).expect("winner missing");
        assert_eq!(winner.agent_id, remote_agent_id, "remote agent should win");
        assert!(
            !bundle.contains(task_id),
            "task must be removed from local bundle after loss"
        );
    }

    // Test that the local winner has a higher bid than
    // the incoming remote winner for the same task,
    // so process_gossip must leave both the bundle and winners unchanged.
    #[test]
    fn lower_bid_does_not_override_local_winner() {
        let local_agent_id = 1000;
        let remote_agent_id = 1001;
        let task_id = 2000;

        let bid = 12.0;
        let higher_bid = bid + 12.0;

        let local_ts = 100;
        let remote_ts = 200;

        let (mut bundle, mut winners) = setup();

        bundle.insert(task_id);
        winners.insert(task_id, local_agent_id, higher_bid, local_ts);

        let gossip = CbbaGossip {
            agent_id: remote_agent_id,
            winners: vec![Winner {
                task_id,
                agent_id: remote_agent_id,
                bid,
                ts: remote_ts,
            }],
        };

        process_gossip(local_agent_id, &mut bundle, &mut winners, &gossip);

        assert!(bundle.contains(task_id), "Task must remain in bundle");

        let winner = winners.get(task_id).expect("Winner must exist");
        assert_eq!(winner.agent_id, local_agent_id);
        assert_eq!(winner.bid, higher_bid);
    }

    // Test newer timestamp wins for equal bid.
    #[test]
    fn newer_timestamp_wins_for_equal_bids() {
        let local_agent_id = 1000;
        let remote_agent_id = 1001;
        let task_id = 2000;

        let bid = 12.0;

        let local_ts = 100;
        let remote_ts = 200;

        let (mut bundle, mut winners) = setup();

        bundle.insert(task_id);
        winners.insert(task_id, local_agent_id, bid, local_ts);

        let gossip = CbbaGossip {
            agent_id: remote_agent_id,
            winners: vec![Winner {
                task_id,
                agent_id: remote_agent_id,
                bid,
                ts: remote_ts,
            }],
        };

        let changed = process_gossip(local_agent_id, &mut bundle, &mut winners, &gossip);
        assert!(changed, "Newer timestamp with equal bids must change state");

        assert!(!bundle.contains(task_id), "Task must ne removed");

        let winner = winners.get(task_id).expect("Winner must exist");
        assert_eq!(winner.agent_id, remote_agent_id);
        assert!(winner.ts >= remote_ts);
    }

    // Test the lower agent_id wins when bid and timestamp are equal
    #[test]
    fn lower_agent_id_wins_for_equal_bids() {
        let local_agent_id = 1000;
        let remote_agent_id = 1001;
        let task_id = 2000;

        let bid = 12.0;
        let ts = 100;

        let (mut bundle, mut winners) = setup();

        bundle.insert(task_id);
        winners.insert(task_id, local_agent_id, bid, ts);

        let gossip = CbbaGossip {
            agent_id: remote_agent_id,
            winners: vec![Winner {
                task_id,
                agent_id: remote_agent_id,
                bid,
                ts,
            }],
        };

        process_gossip(local_agent_id, &mut bundle, &mut winners, &gossip);

        assert!(bundle.contains(task_id), "Task remain in the local bundle");

        let winner = winners.get(task_id).expect("Winner must exist");
        assert_eq!(winner.agent_id, local_agent_id);
    }

    // Test incoming knowledge is adopted when the agent has no local opinion.
    #[test]
    fn incoming_knowledge_is_adopted_when_no_local_opinion() {
        let local_agent_id = 1000;
        let remote_agent_id = 1001;
        let task_id = 2000;

        let bid = 12.0;
        let ts = 100;

        let (mut bundle, mut winners) = setup();

        let gossip = CbbaGossip {
            agent_id: remote_agent_id,
            winners: vec![Winner {
                task_id,
                agent_id: remote_agent_id,
                bid,
                ts,
            }],
        };

        let changed = process_gossip(local_agent_id, &mut bundle, &mut winners, &gossip);

        assert!(changed, "A new winner for local agent must change state");
        assert!(
            !bundle.contains(task_id),
            "Task must NOT be in the local bundle"
        );

        let winner = winners.get(task_id).expect("Winner must exist");
        assert_eq!(winner.agent_id, remote_agent_id);
    }

    // Return false if the gossip came from the local agent.
    #[test]
    fn processing_local_gossip_does_not_change_state() {
        let local_agent_id = 1000;
        let remote_agent_id = 1001;
        let task_id = 2000;

        let bid = 12.0;
        let ts = 100;

        let (mut bundle, mut winners) = setup();

        let gossip = CbbaGossip {
            agent_id: local_agent_id,
            winners: vec![Winner {
                task_id,
                agent_id: remote_agent_id,
                bid,
                ts,
            }],
        };

        let changed = process_gossip(local_agent_id, &mut bundle, &mut winners, &gossip);
        assert!(!changed, "State must NOT change");
    }
}
