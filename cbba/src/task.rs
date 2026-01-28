use crate::Location;

// const TASK_COUNT_WEIGHT: f64 = 0.75;
//
pub type TaskId = u32;

pub struct TaskContext {
    pub task_count: usize,
    pub agent_location: Location,
    pub energy: f64,
    pub task_count_weight: f64,
}

pub trait Task {
    fn id(&self) -> TaskId;
    fn ts(&self) -> u64;

    // Calculates the task bid based on the provided context
    fn calculate_task_bid(&self, ctx: &TaskContext) -> f64;
}

pub struct DefaultTask {
    pub id: TaskId,
    pub ts: u64,
    pub location: Location,
    pub priority: u16,
}

impl Task for DefaultTask {
    fn id(&self) -> TaskId {
        self.id
    }
    fn ts(&self) -> u64 {
        self.ts
    }

    fn calculate_task_bid(&self, ctx: &TaskContext) -> f64 {
        let distance = ctx.agent_location.distance_to(&self.location);
        let distance_score = 1.0 + distance;
        let task_penalty = 1.0 + ctx.task_count as f64 * ctx.task_count_weight;
        let priority = self.priority as f64;
        (ctx.energy * priority) / (task_penalty * distance_score)
    }
}
