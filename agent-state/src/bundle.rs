use crate::task::TaskId;
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Serialize)]
pub struct Bundle {
    task_ids: Vec<TaskId>,
}

pub type SharedBundle = Arc<RwLock<Bundle>>;

impl Bundle {
    pub fn new() -> Self {
        Self {
            task_ids: Vec::new(),
        }
    }

    pub fn remove(&mut self, task_id: TaskId) {
        if let Some(pos) = self.task_ids.iter().position(|&t| t == task_id) {
            self.task_ids.remove(pos);
        }
    }

    pub fn contains(&self, task_id: TaskId) -> bool {
        self.task_ids.contains(&task_id)
    }

    pub fn insert(&mut self, task_id: TaskId) {
        if !self.contains(task_id) {
            self.task_ids.push(task_id);
        }
    }

    pub fn replace(&mut self, task_ids: Vec<TaskId>) {
        self.task_ids.clear();
        self.task_ids.extend(task_ids);
    }

    pub fn truncate_after(&mut self, lost_task: TaskId) {
        if let Some(pos) = self.task_ids.iter().position(|&t| t == lost_task) {
            self.task_ids.truncate(pos);
        }
    }

    pub fn task_ids(&self) -> Vec<TaskId> {
        self.task_ids.iter().cloned().collect()
    }

    pub fn clear(&mut self) {
        self.task_ids.clear();
    }

    pub fn len(&self) -> usize {
        self.task_ids.len()
    }
}
