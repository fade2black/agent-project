use crate::server::StateServerError;
use agent_state::SharedAgentState;
use agent_state::Task;
use serde::Serialize;
use std::sync::Arc;

#[derive(Serialize)]
pub(super) struct TasksResponse {
    tasks_count: usize,
    tasks: Vec<Task>,
}

pub(super) async fn handler(
    state: Arc<SharedAgentState>,
) -> Result<TasksResponse, StateServerError> {
    let store = state.task_store.read().await;
    let tasks = store.get_tasks();

    let tasks_count = tasks.len();

    Ok(TasksResponse { tasks_count, tasks })
}
