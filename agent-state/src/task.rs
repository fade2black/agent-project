use crate::Location;
use serde::{Deserialize, Serialize};

pub type TaskId = u32;

pub struct TaskContext {
    pub task_count: usize,
    pub agent_location: Location,
    pub energy: f64,
    pub task_count_weight: f64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Task {
    pub id: TaskId,
    pub ts: u64,
    pub location: Location,
    pub priority: u16,
}

impl Task {
    pub fn calculate_task_bid(&self, ctx: &TaskContext) -> f64 {
        let distance = ctx.agent_location.distance_to(&self.location);
        let distance_score = 1.0 + distance;
        let task_penalty = 1.0 + ctx.task_count as f64 * ctx.task_count_weight;
        let priority = self.priority as f64;
        (ctx.energy * priority) / (task_penalty * distance_score)
    }
}
