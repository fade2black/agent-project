use common::time::now;
use std::collections::HashMap;
use std::net::IpAddr;
use tracing::info;

#[derive(Clone)]
pub struct AgentEntry {
    id: u32,
    addr: IpAddr,
    last_seen: u64,
}

impl AgentEntry {
    pub fn new(id: u32, addr: IpAddr, last_seen: u64) -> Self {
        AgentEntry {
            id,
            addr,
            last_seen,
        }
    }

    fn is_alive(&self, ttl: u64) -> bool {
        now().saturating_sub(self.last_seen) < ttl
    }
}

pub(crate) struct AgentStore {
    store: HashMap<u32, AgentEntry>,
}

impl AgentStore {
    pub fn new() -> Self {
        AgentStore {
            store: HashMap::new(),
        }
    }

    // Cleanup expired sessions
    pub fn cleanup(&mut self, ttl: u64) {
        self.store.retain(|_, agent| agent.is_alive(ttl));

        info!(
            "Agent store cleaned up (total {} agents remaining).",
            self.store.len()
        );
    }

    pub fn insert(&mut self, agent_id: u32, addr: IpAddr) {
        let agent = AgentEntry::new(agent_id, addr, now());
        self.store.insert(agent_id, agent);

        info!("Agent with id {agent_id} added.");
    }

    pub fn get_alive_agents(&self, ttl: u64) -> Vec<AgentEntry> {
        self.store
            .values()
            .filter(|agent| agent.is_alive(ttl))
            .cloned()
            .collect()
    }
}
