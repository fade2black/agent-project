use crate::server::AgentState;
use serde::Serialize;
use udp_discovery::AgentEntry;

#[derive(Serialize)]
pub(super) struct AgentsResponse {
    agents_count: usize,
    agents: Vec<AgentEntry>,
}

pub(super) async fn handler(
    state: &AgentState,
) -> Result<AgentsResponse, Box<dyn std::error::Error>> {
    let agents = state.discovery.get_alive_agents();
    let agents_count = agents.len();

    Ok(AgentsResponse {
        agents_count,
        agents,
    })
}
