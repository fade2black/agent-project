use common::get_env_var;
use serde::Serialize;

#[derive(Clone, Copy, Serialize)]
pub struct Config {
    pub agent_id: u32,
    pub agent_ttl: u64,
    pub discovery_interval: u64,
    pub discovery_port: u16,
    pub cbba_port: u16,
    pub cbba_timeout: u64,
    pub command_control_port: u16,
    pub agent_cleanup_interval: u64,
    pub http_port: u16,
}

impl Config {
    pub fn new() -> Self {
        Self {
            agent_id: get_env_var("AGENT_ID"),
            agent_ttl: get_env_var("AGENT_TTL"),
            cbba_port: get_env_var("CBBA_PORT"),
            cbba_timeout: get_env_var("CBBA_TIMEOUT"),
            command_control_port: get_env_var("COMMAND_CONTROL_PORT"),
            discovery_interval: get_env_var("DISCOVERY_INTERVAL"),
            discovery_port: get_env_var("DISCOVERY_PORT"),
            agent_cleanup_interval: get_env_var("AGENT_CLEANUP_INTERVAL"),
            http_port: get_env_var("HTTP_PORT"),
        }
    }
}
