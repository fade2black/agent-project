use agent_state::{
    Bundle, CbbaGossip, Config, Location, Task, TaskContext, TaskStore, Winner, Winners,
};

use bytes::BytesMut;
use common::RmpSerializable;
use common::time::now;
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
}

enum ConflictDecision {
    RemoteWins,
    LocalWins,
}

pub async fn start() -> Result<(), CbbaError> {
    let config = Config::new();
    let port = config.cbba_port;
    let mut bytes = BytesMut::new();

    init();

    let mut sender = UdpTransport::new_sender(port).await?;
    let mut receiver = UdpTransport::new_receiver(port).await?;

    let timer = time::sleep(Duration::from_secs(config.cbba_timeout));
    tokio::pin!(timer);

    info!("⚙️ Starting CBBA phase...");

    //Send initial gossip
    send_gossip(&mut sender).await;

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
                if process_gossip(&gossip) {
                    //send gossip
                } else {
                    info!("No changes in bundle or winners.");
                }

            },
            _ = &mut timer => {
                info!("Timeout reached");
                break Ok(());
            }
        }
    }
}

/// If bundle or winners changed, return true, false otherwise
fn process_gossip(gossip: &CbbaGossip) -> bool {
    // info!("Processing Gossip: {:?}", gossip);

    // if self.agent_id == gossip.agent_id {
    //     return false;
    // }

    // let mut bundle_changed = false;

    // // Loop through the winners in the received gossip
    // for remote in gossip.winners.iter() {
    //     let task_id = remote.task_id;

    //     if let Some(local) = self.winners.get(task_id) {
    //         if remote.agent_id == self.agent_id {
    //             continue; // Dude! We've already agreed!
    //         }

    //         match self.compare(&remote, &local) {
    //             ConflictDecision::RemoteWins => {
    //                 self.bundle.remove(task_id);
    //                 self.winners
    //                     .insert(task_id, remote.agent_id, remote.bid, now());
    //             }
    //             ConflictDecision::LocalWins => {
    //                 self.bundle.insert(task_id);
    //                 self.winners
    //                     .insert(task_id, local.agent_id, local.bid, now());
    //             }
    //         }

    //         bundle_changed = true;
    //     } else {
    //         info!(
    //             "Agent does not have a local winner for the task {}. Potential inconsistency in the task store.",
    //             task_id
    //         );
    //         self.winners
    //             .insert(task_id, remote.agent_id, remote.bid, now());
    //         bundle_changed = true;
    //     }
    // }

    // bundle_changed
    true
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

fn init() {
    let ctx = create_task_context();

    // Compute initial bids and initialize the bundle and winners

    // let bids = tasks_store.compute_local_bids(&ctx);
    // bundle.init(bids.keys().cloned().collect());
    // winners.init(1000, bids);
}

fn create_task_context() -> TaskContext {
    TaskContext {
        task_count_weight: 0.75,
        task_count: 0, //self.tasks_store.tasks_count(),
        agent_location: Location { lat: 0.0, lon: 0.0 },
        energy: 100.0,
    }
}

async fn send_gossip(sender: &mut UdpTransport) {
    // let gossip = self.winners.to_gossip(self.agent_id);
    // info!("Sending Gossip: {:?}", gossip);

    // let bytes = gossip.to_bytes().unwrap();
    // sender.send(&bytes).await.unwrap();
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::DefaultTask;

//     fn create_cbbs(agent_id: u32) -> CbbaPhase<DefaultTask> {
//         CbbaPhase::new(agent_id, 4001, 10)
//     }

//     // Test that the state has changed because the local agent wins the task.
//     // Check that the winner for the task changes to the local agent.
//     // Ensure that the task has been added to the local bundle.
//     #[test]
//     fn local_higher_bid_wins() {
//         let local_agent_id = 1000;
//         let remote_agent_id = 1001;

//         let task_id = 2000;
//         let bid = 10.0;
//         let lower_bid = bid - 1.0;
//         let ts = 10;

//         let mut cbba = create_cbbs(local_agent_id);
//         cbba.winners.insert(task_id, local_agent_id, bid, ts);

//         // Remote gossip
//         let gossip = CbbaGossip {
//             agent_id: remote_agent_id,
//             winners: vec![Winner {
//                 task_id,
//                 agent_id: remote_agent_id, // another agent
//                 bid: lower_bid,            // Lower bid
//                 ts,                        // Same timestamp
//             }],
//         };

//         let changed = cbba.process_gossip(&gossip);
//         assert!(changed, "state should be marked as changed");

//         let winner = cbba.winners.get(task_id).expect("winner missing");
//         assert_eq!(winner.agent_id, local_agent_id, "remote agent should win");
//         assert!(
//             cbba.bundle.contains(task_id),
//             "task must be added to local bundle after win"
//         );
//     }

//     // Test that the state has changed because the local agent lost the task.
//     // Check that the winner for the task changes to the remote agent.
//     // Ensure that the task has been removed from the local bundle.
//     #[test]
//     fn remote_higher_bid_wins() {
//         let local_agent_id = 1000;
//         let remote_agent_id = 1001;

//         let task_id = 2000;
//         let bid = 10.0;
//         let higher_bid = bid + 5.0;
//         let ts = 10; // timestamp

//         let mut cbba = create_cbbs(local_agent_id);

//         cbba.bundle.init(vec![task_id]);
//         cbba.winners.insert(task_id, local_agent_id, bid, ts);

//         // Remote gossip
//         let gossip = CbbaGossip {
//             agent_id: remote_agent_id,
//             winners: vec![Winner {
//                 task_id,
//                 agent_id: remote_agent_id, // another agent
//                 bid: higher_bid,           // Higher bid
//                 ts,                        // Same timestamp
//             }],
//         };

//         let changed = cbba.process_gossip(&gossip);
//         assert!(changed, "state should be marked as changed");

//         let winner = cbba.winners.get(task_id).expect("winner missing");
//         assert_eq!(winner.agent_id, remote_agent_id, "remote agent should win");
//         assert!(
//             !cbba.bundle.contains(task_id),
//             "task must be removed from local bundle after loss"
//         );
//     }

//     // Test that the local winner has a higher bid than
//     // the incoming remote winner for the same task,
//     // so process_gossip must leave both the bundle and winners unchanged.
//     #[test]
//     fn lower_bid_does_not_override_local_winner() {
//         let local_agent_id = 1000;
//         let remote_agent_id = 1001;
//         let task_id = 2000;

//         let bid = 12.0;
//         let higher_bid = bid + 12.0;

//         let local_ts = 100;
//         let remote_ts = 200;

//         let mut cbba = create_cbbs(local_agent_id);

//         cbba.bundle.insert(task_id);
//         cbba.winners
//             .insert(task_id, local_agent_id, higher_bid, local_ts);

//         let gossip = CbbaGossip {
//             agent_id: remote_agent_id,
//             winners: vec![Winner {
//                 task_id,
//                 agent_id: remote_agent_id,
//                 bid,
//                 ts: remote_ts,
//             }],
//         };

//         cbba.process_gossip(&gossip);

//         assert!(cbba.bundle.contains(task_id), "Task must remain in bundle");

//         let winner = cbba.winners.get(task_id).expect("Winner must exist");
//         assert_eq!(winner.agent_id, local_agent_id);
//         assert_eq!(winner.bid, higher_bid);
//     }

//     // Test newer timestamp wins for equal bid.
//     #[test]
//     fn newer_timestamp_wins_for_equal_bids() {
//         let local_agent_id = 1000;
//         let remote_agent_id = 1001;
//         let task_id = 2000;

//         let bid = 12.0;

//         let local_ts = 100;
//         let remote_ts = 200;

//         let mut cbba = create_cbbs(local_agent_id);
//         cbba.bundle.insert(task_id);
//         cbba.winners.insert(task_id, local_agent_id, bid, local_ts);

//         let gossip = CbbaGossip {
//             agent_id: remote_agent_id,
//             winners: vec![Winner {
//                 task_id,
//                 agent_id: remote_agent_id,
//                 bid,
//                 ts: remote_ts,
//             }],
//         };

//         let changed = cbba.process_gossip(&gossip);
//         assert!(changed, "Newer timestamp with equal bids must change state");

//         assert!(!cbba.bundle.contains(task_id), "Task must ne removed");

//         let winner = cbba.winners.get(task_id).expect("Winner must exist");
//         assert_eq!(winner.agent_id, remote_agent_id);
//         assert!(winner.ts >= remote_ts);
//     }

//     // Test the lower agent_id wins when bid and timestamp are equal
//     #[test]
//     fn lower_agent_id_wins_for_equal_bids() {
//         let local_agent_id = 1000;
//         let remote_agent_id = 1001;
//         let task_id = 2000;

//         let bid = 12.0;
//         let ts = 100;

//         let mut cbba = create_cbbs(local_agent_id);
//         cbba.bundle.insert(task_id);
//         cbba.winners.insert(task_id, local_agent_id, bid, ts);

//         let gossip = CbbaGossip {
//             agent_id: remote_agent_id,
//             winners: vec![Winner {
//                 task_id,
//                 agent_id: remote_agent_id,
//                 bid,
//                 ts,
//             }],
//         };

//         cbba.process_gossip(&gossip);

//         assert!(
//             cbba.bundle.contains(task_id),
//             "Task remain in the local bundle"
//         );

//         let winner = cbba.winners.get(task_id).expect("Winner must exist");
//         assert_eq!(winner.agent_id, local_agent_id);
//     }

//     // Test incoming knowledge is adopted when the agent has no local opinion.
//     #[test]
//     fn incoming_knowledge_is_adopted_when_no_local_opinion() {
//         let local_agent_id = 1000;
//         let remote_agent_id = 1001;
//         let task_id = 2000;

//         let bid = 12.0;
//         let ts = 100;

//         let mut cbba = create_cbbs(local_agent_id);

//         let gossip = CbbaGossip {
//             agent_id: remote_agent_id,
//             winners: vec![Winner {
//                 task_id,
//                 agent_id: remote_agent_id,
//                 bid,
//                 ts,
//             }],
//         };

//         let changed = cbba.process_gossip(&gossip);

//         assert!(changed, "A new winner for local agent must change state");
//         assert!(
//             !cbba.bundle.contains(task_id),
//             "Task must NOT be in the local bundle"
//         );

//         let winner = cbba.winners.get(task_id).expect("Winner must exist");
//         assert_eq!(winner.agent_id, remote_agent_id);
//     }

//     // Return false if the gossip came from the local agent.
//     #[test]
//     fn processing_local_gossip_does_not_change_state() {
//         let local_agent_id = 1000;
//         let remote_agent_id = 1001;
//         let task_id = 2000;

//         let bid = 12.0;
//         let ts = 100;

//         let mut cbba = create_cbbs(local_agent_id);

//         let gossip = CbbaGossip {
//             agent_id: local_agent_id,
//             winners: vec![Winner {
//                 task_id,
//                 agent_id: remote_agent_id,
//                 bid,
//                 ts,
//             }],
//         };

//         let changed = cbba.process_gossip(&gossip);
//         assert!(!changed, "State must NOT change");
//     }
// }
