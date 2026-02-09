use crate::task::TaskId;
use std::collections::HashSet;

pub struct Bundle {
    tasks: HashSet<TaskId>,
}

impl Bundle {
    pub fn new() -> Self {
        Self {
            tasks: HashSet::new(),
        }
    }

    pub fn init(&mut self, task_ids: Vec<TaskId>) {
        self.tasks.clear();
        self.tasks.extend(task_ids);
    }

    pub fn remove(&mut self, task_id: TaskId) -> bool {
        self.tasks.remove(&task_id)
    }

    #[cfg(test)]
    pub fn contains(&self, task_id: TaskId) -> bool {
        self.tasks.contains(&task_id)
    }

    pub fn insert(&mut self, task_id: TaskId) {
        self.tasks.insert(task_id);
    }
}
