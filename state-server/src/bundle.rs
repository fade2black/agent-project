use crate::server::StateServerError;
use agent_state::SharedAgentState;
use agent_state::TaskId;
use serde::Serialize;
use std::sync::Arc;

#[derive(Serialize)]
pub(super) struct BundleResponse {
    count: usize,
    task_ids: Vec<TaskId>,
}

pub(super) async fn handler(
    state: Arc<SharedAgentState>,
) -> Result<BundleResponse, StateServerError> {
    let bundle = state.bundle.read().await;
    let task_ids = bundle.task_ids();
    let count = task_ids.len();

    Ok(BundleResponse { count, task_ids })
}
