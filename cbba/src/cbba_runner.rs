use agent_state::{
    Bundle, CbbaGossip, Config, Location, SharedAgentState, TaskContext, Winner, Winners,
};
use bytes::BytesMut;
use common::time::now;
use common::{RmpSerializable, SerializationError};
use std::sync::Arc;
use thiserror::Error;
use tokio::time::{self, Duration};
use tracing::{error, info, warn};
use transport::Transport;
use udp_transport::UdpTransport;

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
    agent_state: Arc<SharedAgentState>,
}

impl CbbaRunner {
    pub fn new(config: Config, agent_state: Arc<SharedAgentState>) -> Self {
        Self {
            agent_state,
            config,
        }
    }
}

impl CbbaRunner {
    pub async fn start(&self) -> Result<(), CbbaError> {
        let port = self.config.cbba_port;
        let cbba_timeout = self.config.cbba_timeout;
        let agent_id = self.config.agent_id;

        let (mut bundle, mut winners) = self.create_cbb_state().await;
        let mut sender = UdpTransport::new_sender(port).await?;
        let mut receiver = UdpTransport::new_receiver(port).await?;
        let mut bytes = BytesMut::new();

        let timer = time::sleep(Duration::from_secs(cbba_timeout));
        tokio::pin!(timer);

        info!("⚙️ Starting CBBA process...");

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

                    info!("Received gossip ({} bytes).", size);

                    let Ok(gossip) = CbbaGossip::from_bytes(&bytes[..size]) else {
                        warn!("Dropping invalid gossip.");
                        continue;
                    };

                    info!("Received gossip: {:?}", gossip);
                    if self.process_gossip(&mut bundle, &mut winners, &gossip) {
                        let gossip = winners.to_gossip(agent_id);
                        send_gossip(&mut sender, &gossip).await?;
                    } else {
                        info!("No changes in bundle or winners.");
                    }
                },
                _ = &mut timer => {
                    info!("Timeout reached.");
                    break;
                }
            }
        }

        // Update the agent state with the new bundle and winners
        let mut prev_bundle = self.agent_state.bundle.write().await;
        let mut prev_winners = self.agent_state.winners.write().await;
        *prev_bundle = bundle;
        *prev_winners = winners;

        Ok(())
    }

    async fn create_cbb_state(&self) -> (Bundle, Winners) {
        let mut bundle = Bundle::new();
        let mut winners = Winners::new();

        let task_store = self.agent_state.task_store.read().await;
        let ctx = create_task_context(task_store.tasks_count());

        // Compute initial bids and initialize the bundle and winners
        let bids = task_store.compute_local_bids(&ctx);
        bundle.replace(bids.keys().cloned().collect());
        winners.init(self.config.agent_id, bids);

        (bundle, winners)
    }

    /// If bundle or winners changed, return true, false otherwise
    fn process_gossip(
        &self,
        bundle: &mut Bundle,
        winners: &mut Winners,
        gossip: &CbbaGossip,
    ) -> bool {
        let agent_id = self.config.agent_id;

        info!("Processing Gossip: {:?}", gossip);

        if agent_id == gossip.agent_id {
            return false;
        }

        let mut bundle_changed = false;

        // Loop through the winners in the received gossip
        for remote in gossip.winners.iter() {
            let task_id = remote.task_id;

            if let Some(local) = winners.get(task_id) {
                if remote.agent_id == agent_id {
                    continue; // Dude! We've already agreed!
                }

                match compare(&remote, &local) {
                    ConflictDecision::RemoteWins => {
                        bundle.remove(task_id);
                        winners.insert(task_id, remote.agent_id, remote.bid, now());
                    }
                    ConflictDecision::LocalWins => {
                        bundle.insert(task_id);
                        winners.insert(task_id, local.agent_id, local.bid, now());
                    }
                }

                bundle_changed = true;
            } else {
                info!(
                    "Agent does not have a local winner for the task {}. Potential inconsistency in the task store.",
                    task_id
                );
                winners.insert(task_id, remote.agent_id, remote.bid, now());
                bundle_changed = true;
            }
        }

        bundle_changed
    }
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

fn create_task_context(task_count: usize) -> TaskContext {
    TaskContext {
        task_count,
        task_count_weight: 0.75,
        agent_location: Location { lat: 0.0, lon: 0.0 },
        energy: 100.0,
    }
}

async fn send_gossip(sender: &mut UdpTransport, gossip: &CbbaGossip) -> Result<(), CbbaError> {
    let bytes = gossip.to_bytes()?;
    info!("Sending Gossip: {:?}", gossip);
    sender.send(&bytes).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config(agent_id: u32) -> Config {
        Config {
            agent_id,
            agent_ttl: 10,
            discovery_interval: 10,
            discovery_port: 4000,
            cbba_port: 4001,
            cbba_timeout: 30,
            command_control_port: 4002,
            agent_cleanup_interval: 10,
            http_port: 8000,
        }
    }

    fn setup(agent_id: u32) -> (CbbaRunner, Bundle, Winners) {
        let bundle = Bundle::new();
        let winners = Winners::new();
        let config = create_test_config(agent_id);
        let agent_state = Arc::new(SharedAgentState::new(config.agent_ttl));
        let runner = CbbaRunner::new(config, agent_state);

        (runner, bundle, winners)
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

        let (cbba_runner, mut bundle, mut winners) = setup(local_agent_id);

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

        let changed = cbba_runner.process_gossip(&mut bundle, &mut winners, &gossip);
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

        let (cbba_runner, mut bundle, mut winners) = setup(local_agent_id);

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

        let changed = cbba_runner.process_gossip(&mut bundle, &mut winners, &gossip);
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

        let (cbba_runner, mut bundle, mut winners) = setup(local_agent_id);

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

        cbba_runner.process_gossip(&mut bundle, &mut winners, &gossip);

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

        let (cbba_runner, mut bundle, mut winners) = setup(local_agent_id);

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

        let changed = cbba_runner.process_gossip(&mut bundle, &mut winners, &gossip);
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

        let (cbba_runner, mut bundle, mut winners) = setup(local_agent_id);

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

        cbba_runner.process_gossip(&mut bundle, &mut winners, &gossip);

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

        let (cbba_runner, mut bundle, mut winners) = setup(local_agent_id);

        let gossip = CbbaGossip {
            agent_id: remote_agent_id,
            winners: vec![Winner {
                task_id,
                agent_id: remote_agent_id,
                bid,
                ts,
            }],
        };

        let changed = cbba_runner.process_gossip(&mut bundle, &mut winners, &gossip);

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

        let (cbba_runner, mut bundle, mut winners) = setup(local_agent_id);

        let gossip = CbbaGossip {
            agent_id: local_agent_id,
            winners: vec![Winner {
                task_id,
                agent_id: remote_agent_id,
                bid,
                ts,
            }],
        };

        let changed = cbba_runner.process_gossip(&mut bundle, &mut winners, &gossip);
        assert!(!changed, "State must NOT change");
    }
}
