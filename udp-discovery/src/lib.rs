pub mod agent_store;

pub use agent_store::AgentEntry;
use agent_store::AgentStore;
use common::RmpSerializable;
use if_addrs::{IfAddr, get_if_addrs};
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use std::sync::Arc;
use thiserror::Error;
use tokio::{sync::Mutex, task::JoinSet, time::Duration};
use tracing::{error, info};
use transport::Transport;
use udp_transport::UdpTransport;

const MAX_HEARTBEAT_SIZE: usize = 256;

#[derive(Debug, Error)]
pub enum DiscoveryError {
    #[error("I/O error")]
    Io(#[from] std::io::Error),
    #[error("Transport error: {0}")]
    Transport(#[from] udp_transport::TransportError),
    #[error("Serialization error")]
    Serialization(#[from] common::SerializationError),
}

#[derive(Serialize, Deserialize, Debug)]
struct HeartbeatMessage {
    agent_id: u32,
    addr: IpAddr,
}

impl HeartbeatMessage {
    pub fn new(agent_id: u32, addr: IpAddr) -> Self {
        Self { agent_id, addr }
    }
}

#[derive(Clone, Copy)]
pub struct Config {
    interval_sec: u64,
    port: u16,
    agent_ttl_sec: u64,
    agent_cleanup_interval_sec: u64,
}

impl Config {
    pub fn new(
        interval_sec: u64,
        port: u16,
        agent_ttl_sec: u64,
        agent_cleanup_interval_sec: u64,
    ) -> Self {
        Self {
            interval_sec,
            port,
            agent_ttl_sec,
            agent_cleanup_interval_sec,
        }
    }
}

pub struct UdpDiscovery {
    agent_id: u32,
    config: Config,
    agent_store: Arc<Mutex<AgentStore>>,
}

impl UdpDiscovery {
    pub fn new(agent_id: u32, config: Config) -> Self {
        let agent_store = Arc::new(Mutex::new(AgentStore::new()));

        Self {
            agent_id,
            config,
            agent_store,
        }
    }

    pub async fn get_alive_agents(&self) -> Vec<AgentEntry> {
        let ttl = self.config.agent_ttl_sec;
        let store = self.agent_store.lock().await;
        store.get_alive_agents(ttl)
    }

    pub async fn add_agent(&self, agent_id: u32, addr: IpAddr) {
        let mut store = self.agent_store.lock().await;
        store.insert(agent_id, addr);
    }

    pub async fn cleanup(&self) {
        let ttl = self.config.agent_ttl_sec;
        let mut store = self.agent_store.lock().await;

        store.cleanup(ttl);
    }

    pub async fn start(self: Arc<Self>) {
        let mut tasks = JoinSet::new();

        tasks.spawn(send_heartbeat_task(self.clone()));
        tasks.spawn(recv_heartbeat_task(self.clone()));
        tasks.spawn(cleanup_task(self.clone()));

        while let Some(res) = tasks.join_next().await {
            match res {
                Ok(Err(e)) => error!("Error in task: {}", e),
                Ok(Ok(())) => info!("Heartbeat task completed successfully"),
                Err(e) => error!("Task panicked: {}", e),
            }
        }
    }
}

async fn send_heartbeat_task(discovery: Arc<UdpDiscovery>) -> Result<(), DiscoveryError> {
    let config = discovery.config;

    let ips = retrieve_usable_ips()?;
    let duration = Duration::from_secs(config.interval_sec);
    let mut sender = UdpTransport::new_sender(config.port).await?;

    info!("Starting sending heartbeat task...");
    info!(
        "Port {}, Interval: {} seconds.",
        config.port, config.interval_sec
    );

    loop {
        let msg = HeartbeatMessage::new(discovery.agent_id, ips[0]);
        let bytes = msg.to_bytes()?;

        // TODO: specific error handling for send() could be beneficial for debuging
        sender.send(&bytes).await?;
        info!("Heartbeat message sent.");

        tokio::time::sleep(duration).await;
    }
}

async fn recv_heartbeat_task(discovery: Arc<UdpDiscovery>) -> Result<(), DiscoveryError> {
    let port = discovery.config.port;

    let mut receiver = UdpTransport::new_receiver(port).await?;
    let mut buffer = [0u8; MAX_HEARTBEAT_SIZE];

    info!("Starting receiving heartbeat task...");
    loop {
        match receiver.recv(&mut buffer).await? {
            Some(size) => {
                let msg = HeartbeatMessage::from_bytes(&buffer[..size])?;

                if msg.agent_id != discovery.agent_id {
                    info!("Received heartbeat {} bytes from {}", size, msg.agent_id);
                    discovery.add_agent(msg.agent_id, msg.addr).await;
                }
            }
            None => {
                info!("No packets received.");
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    }
}

async fn cleanup_task(discovery: Arc<UdpDiscovery>) -> Result<(), DiscoveryError> {
    let interval = discovery.config.agent_cleanup_interval_sec;
    let duration = Duration::from_secs(interval);

    info!("Starting agent store cleanup task...");

    loop {
        tokio::time::sleep(duration).await;
        discovery.cleanup().await;
    }
}

pub fn retrieve_usable_ips() -> Result<Vec<IpAddr>, DiscoveryError> {
    let ifaces = get_if_addrs()?;

    Ok(ifaces
        .into_iter()
        .filter_map(|iface| match iface.addr {
            IfAddr::V4(v4)
                if !v4.ip.is_loopback() && !v4.ip.is_link_local() && !iface.is_loopback() =>
            {
                Some(IpAddr::V4(v4.ip))
            }
            _ => None,
        })
        .collect())
}
