use crate::{Task, TaskContext, TaskId};
use std::collections::{HashMap, hash_map::Entry};
use tracing::info;

pub struct TaskStore<T: Task> {
    tasks: HashMap<TaskId, T>,
}

impl<T> TaskStore<T>
where
    T: Task,
{
    pub(crate) fn new() -> Self {
        Self {
            tasks: HashMap::new(),
        }
    }

    pub(crate) fn insert_task(&mut self, task: T) {
        if let Entry::Occupied(mut e) = self.tasks.entry(task.id()) {
            if task.ts() > e.get().ts() {
                e.insert(task);
            } else {
                info!("Rejecting outdated task: {}", task.id());
            }
        } else {
            self.tasks.insert(task.id(), task);
        }
    }

    pub(crate) fn insert_tasks(&mut self, tasks: Vec<T>) {
        for task in tasks {
            self.insert_task(task);
        }
    }

    pub(crate) fn remove_task(&mut self, task_id: u32) -> Option<T> {
        self.tasks.remove(&task_id)
    }

    pub(crate) fn tasks_count(&self) -> usize {
        self.tasks.len()
    }

    pub(crate) fn compute_local_bids(&self, ctx: &TaskContext) -> HashMap<TaskId, f64> {
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

    #[derive(Clone)]
    struct DummyTask {
        id: TaskId,
        ts: u64,
    }

    fn create_task(id: TaskId, ts: u64) -> DummyTask {
        DummyTask { id, ts }
    }

    impl Task for DummyTask {
        fn id(&self) -> u32 {
            self.id
        }

        fn ts(&self) -> u64 {
            self.ts
        }

        fn calculate_task_bid(&self, _: &TaskContext) -> f64 {
            0.0 // Simple dummy calculation
        }
    }

    #[test]
    fn test_insert_task_new() {
        let mut store: TaskStore<DummyTask> = TaskStore::new();
        let task = create_task(1, 10);

        store.insert_task(task);

        assert_eq!(store.tasks.len(), 1);
        assert_eq!(store.tasks.get(&1).unwrap().ts(), 10);
    }

    #[test]
    fn test_insert_task_update() {
        let id = 1;

        let mut store: TaskStore<DummyTask> = TaskStore::new();
        let task1 = create_task(id, 10);
        let task2 = create_task(id, 20);

        store.insert_task(task1);
        store.insert_task(task2); // Insert the second task that should replace the first one

        assert_eq!(store.tasks.len(), 1);
        assert_eq!(store.tasks.get(&id).unwrap().ts(), 20);
    }

    #[test]
    fn test_insert_task_reject_outdated() {
        let id = 1;
        let mut store: TaskStore<DummyTask> = TaskStore::new();
        let task1 = create_task(id, 10);
        let task2 = create_task(id, 5); // New task with an older timestamp

        store.insert_task(task1);
        store.insert_task(task2); // Try inserting the outdated task

        assert_eq!(store.tasks.len(), 1);
        assert_eq!(store.tasks.get(&id).unwrap().ts(), 10);
    }
}
