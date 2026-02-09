use crate::server::{StateServerContext, StateServerError};
use agent_state::Task;
use serde::Serialize;

#[derive(Serialize)]
pub(super) struct TasksResponse {
    tasks_count: usize,
    tasks: Vec<Task>,
}

pub(super) async fn handler(state: &StateServerContext) -> Result<TasksResponse, StateServerError> {
    let store = state.task_store.read().await;
    let tasks = store.get_tasks();

    let tasks_count = tasks.len();

    Ok(TasksResponse { tasks_count, tasks })
}
