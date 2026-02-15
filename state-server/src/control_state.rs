use crate::server::StateServerError;
use agent_state::ControlState;
use agent_state::SharedAgentState;
use serde::Serialize;
use std::sync::Arc;

#[derive(Serialize)]
pub(super) struct ControlStateResponse {
    control_state: ControlState,
}

pub(super) async fn handler(
    state: Arc<SharedAgentState>,
) -> Result<ControlStateResponse, StateServerError> {
    let control_state = *state.control_state.read().await;
    Ok(ControlStateResponse { control_state })
}
