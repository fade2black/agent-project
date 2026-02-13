use crate::{
    AgentStore, Bundle, ControlState, SharedAgentStore, SharedBundle, SharedControlState,
    SharedTaskStore, SharedWinners, TaskStore, Winners,
};
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct SharedAgentState {
    pub agent_store: SharedAgentStore,
    pub task_store: SharedTaskStore,
    pub bundle: SharedBundle,
    pub winners: SharedWinners,
    pub control_state: SharedControlState,
}

impl SharedAgentState {
    pub fn new(agent_ttl: u64) -> Self {
        SharedAgentState {
            agent_store: Arc::new(RwLock::new(AgentStore::new(agent_ttl))),
            task_store: Arc::new(RwLock::new(TaskStore::new())),
            bundle: Arc::new(RwLock::new(Bundle::new())),
            winners: Arc::new(RwLock::new(Winners::new())),
            control_state: Arc::new(RwLock::new(ControlState::new())),
        }
    }
}
