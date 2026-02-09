use crate::server::{StateServerContext, StateServerError};
use agent_state::AgentEntry;
use serde::Serialize;

#[derive(Serialize)]
pub(super) struct AgentsResponse {
    agents_count: usize,
    agents: Vec<AgentEntry>,
}

pub(super) async fn handler(
    state: &StateServerContext,
) -> Result<AgentsResponse, StateServerError> {
    let agent_store = state.agent_store.read().await;
    let agents = agent_store.get_alive_agents();
    let agents_count = agents.len();

    Ok(AgentsResponse {
        agents_count,
        agents,
    })
}
