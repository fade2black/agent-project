use crate::server::StateServerError;
use agent_state::AgentEntry;
use agent_state::SharedAgentState;
use serde::Serialize;
use std::sync::Arc;

#[derive(Serialize)]
pub(super) struct AgentsResponse {
    agents_count: usize,
    agents: Vec<AgentEntry>,
}

pub(super) async fn handler(
    state: Arc<SharedAgentState>,
) -> Result<AgentsResponse, StateServerError> {
    let agent_store = state.agent_store.read().await;
    let agents = agent_store.get_alive_agents();
    let agents_count = agents.len();

    Ok(AgentsResponse {
        agents_count,
        agents,
    })
}
