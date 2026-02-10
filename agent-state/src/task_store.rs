use crate::{Task, TaskContext, TaskId};
use std::collections::{HashMap, hash_map::Entry};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

pub struct TaskStore {
    tasks: HashMap<TaskId, Task>,
}

pub type SharedTaskStore = Arc<RwLock<TaskStore>>;

impl TaskStore {
    pub fn new() -> Self {
        Self {
            tasks: HashMap::new(),
        }
    }

    pub fn insert_task(&mut self, task: Task) {
        if let Entry::Occupied(mut e) = self.tasks.entry(task.id) {
            if task.ts > e.get().ts {
                e.insert(task);
            } else {
                info!("Rejecting outdated task: {}", task.id);
            }
        } else {
            self.tasks.insert(task.id, task);
        }
    }

    pub fn insert_tasks(&mut self, tasks: Vec<Task>) {
        for task in tasks {
            self.insert_task(task);
        }
    }

    pub fn remove_task(&mut self, task_id: u32) -> Option<Task> {
        self.tasks.remove(&task_id)
    }

    pub fn tasks_count(&self) -> usize {
        self.tasks.len()
    }

    pub fn get_tasks(&self) -> Vec<Task> {
        self.tasks.values().cloned().collect()
    }

    pub fn clear(&mut self) {
        self.tasks.clear();
    }

    pub fn compute_local_bids(&self, ctx: &TaskContext) -> HashMap<TaskId, f64> {
        let mut bids = HashMap::new();

        for (task_id, task) in &self.tasks {
            let bid = task.calculate_task_bid(ctx);
            bids.insert(*task_id, bid);
        }

        bids
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Location;

    fn create_task(id: TaskId, ts: u64) -> Task {
        let location = Location::new(0.0, 0.0);
        Task {
            id,
            ts,
            location,
            priority: 0,
        }
    }

    #[test]
    fn test_insert_task_new() {
        let mut store = TaskStore::new();
        let task = create_task(1, 10);

        store.insert_task(task);

        assert_eq!(store.tasks.len(), 1);
        assert_eq!(store.tasks.get(&1).unwrap().ts, 10);
    }

    #[test]
    fn test_insert_task_update() {
        let id = 1;

        let mut store = TaskStore::new();
        let task1 = create_task(id, 10);
        let task2 = create_task(id, 20);

        store.insert_task(task1);
        store.insert_task(task2); // Insert the second task that should replace the first one

        assert_eq!(store.tasks.len(), 1);
        assert_eq!(store.tasks.get(&id).unwrap().ts, 20);
    }

    #[test]
    fn test_insert_task_reject_outdated() {
        let id = 1;
        let mut store = TaskStore::new();
        let task1 = create_task(id, 10);
        let task2 = create_task(id, 5); // New task with an older timestamp

        store.insert_task(task1);
        store.insert_task(task2); // Try inserting the outdated task

        assert_eq!(store.tasks.len(), 1);
        assert_eq!(store.tasks.get(&id).unwrap().ts, 10);
    }
}
