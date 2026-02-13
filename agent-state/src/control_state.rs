use serde::Serialize;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
pub enum ControlState {
    Idle,
    RunningCBBA,
    RunningDistTasks,
}

impl ControlState {
    pub fn new() -> Self {
        ControlState::Idle
    }
}

pub type SharedControlState = Arc<RwLock<ControlState>>;
