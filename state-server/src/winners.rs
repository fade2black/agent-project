use crate::server::StateServerError;
use agent_state::SharedAgentState;
use agent_state::Winner;
use serde::Serialize;
use std::sync::Arc;

#[derive(Serialize)]
pub(super) struct WinnersResponse {
    count: usize,
    winners: Vec<Winner>,
}

pub(super) async fn handler(
    state: Arc<SharedAgentState>,
) -> Result<WinnersResponse, StateServerError> {
    let winners = state.winners.read().await;
    let winners = winners.get_winners();
    let count = winners.len();

    Ok(WinnersResponse { count, winners })
}
