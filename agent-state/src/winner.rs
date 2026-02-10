use crate::task::TaskId;
use common::time;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Winner {
    pub task_id: u32,
    pub agent_id: u32,
    pub bid: f64,
    pub ts: u64, // when the bid was last accepted
}

impl Winner {
    pub(crate) fn new(task_id: u32, agent_id: u32, bid: f64) -> Self {
        let ts = time::now();

        Self {
            task_id,
            agent_id,
            bid,
            ts,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CbbaGossip {
    pub agent_id: u32,
    pub winners: Vec<Winner>,
}

pub struct Winners {
    winners: HashMap<TaskId, Winner>,
}

pub type SharedWinners = Arc<RwLock<Winners>>;

impl Winners {
    pub fn new() -> Self {
        Self {
            winners: HashMap::new(),
        }
    }

    pub fn to_gossip(&self, agent_id: u32) -> CbbaGossip {
        let winners = self.winners.values().cloned().collect();
        CbbaGossip { agent_id, winners }
    }

    pub fn init(&mut self, agent_id: u32, bids: HashMap<u32, f64>) {
        self.winners.clear();

        for (task_id, bid) in bids {
            self.winners
                .insert(task_id, Winner::new(task_id, agent_id, bid));
        }
    }

    pub fn get(&self, task_id: TaskId) -> Option<&Winner> {
        self.winners.get(&task_id)
    }

    pub fn insert(&mut self, task_id: u32, agent_id: u32, bid: f64, ts: u64) {
        self.winners.insert(
            task_id,
            Winner {
                task_id,
                agent_id,
                bid,
                ts,
            },
        );
    }
}
