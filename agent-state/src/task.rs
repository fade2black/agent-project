use crate::Location;
use crate::telemetry;
use serde::{Deserialize, Serialize};

pub type TaskId = u32;

pub struct TaskContext {
    pub task_count: usize,
    pub agent_location: Location,
    pub energy: f64,
    pub task_count_weight: f64,
}

impl TaskContext {
    pub fn new(task_count: usize) -> Self {
        TaskContext {
            task_count,
            agent_location: telemetry::location(),
            energy: telemetry::energy(),
            task_count_weight: 0.75,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Task {
    pub id: TaskId,
    pub ts: u64,
    pub location: Location,
    pub priority: u16,
}
