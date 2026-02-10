use agent_state::{Config, SharedAgentStore};
use common::RmpSerializable;
use if_addrs::{IfAddr, get_if_addrs};
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use thiserror::Error;
use tokio::{task::JoinSet, time::Duration};
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

pub struct DiscoveryServer {
    config: Config,
    agent_store: SharedAgentStore,
}

impl DiscoveryServer {
    pub fn new(config: Config, agent_store: SharedAgentStore) -> Self {
        Self {
            config,
            agent_store,
        }
    }

    pub async fn start(&self) {
        let mut tasks = JoinSet::new();

        tasks.spawn(send_heartbeat_task(self.config));
        tasks.spawn(recv_heartbeat_task(self.config, self.agent_store.clone()));
        tasks.spawn(cleanup_task(self.config, self.agent_store.clone()));

        while let Some(res) = tasks.join_next().await {
            match res {
                Ok(Err(e)) => error!("Error in task: {}", e),
                Ok(Ok(())) => info!("Heartbeat task completed successfully"),
                Err(e) => error!("Task panicked: {}", e),
            }
        }
    }
}

async fn send_heartbeat_task(config: Config) -> Result<(), DiscoveryError> {
    let ips = retrieve_usable_ips()?;
    let duration = Duration::from_secs(config.discovery_interval);
    let mut sender = UdpTransport::new_sender(config.discovery_port).await?;

    info!("Starting sending heartbeat task...");
    info!(
        "Port {}, Interval: {} seconds.",
        config.discovery_port, config.discovery_interval
    );

    loop {
        let msg = HeartbeatMessage::new(config.agent_id, ips[0]);
        let bytes = msg.to_bytes()?;

        // TODO: specific error handling for send() could be beneficial for debuging
        sender.send(&bytes).await?;
        info!("Heartbeat message sent.");

        tokio::time::sleep(duration).await;
    }
}

async fn recv_heartbeat_task(
    config: Config,
    agent_store: SharedAgentStore,
) -> Result<(), DiscoveryError> {
    let port = config.discovery_port;
    let agent_id = config.agent_id;

    let mut receiver = UdpTransport::new_receiver(port).await?;
    let mut buffer = [0u8; MAX_HEARTBEAT_SIZE];

    info!("Starting receiving heartbeat task...");
    loop {
        if let Ok(size) = receiver.recv(&mut buffer).await {
            let msg = HeartbeatMessage::from_bytes(&buffer[..size])?;

            if msg.agent_id == agent_id {
                continue;
            }
            info!("Received heartbeat {} bytes from {}", size, msg.agent_id);

            let mut store = agent_store.write().await;
            store.insert(msg.agent_id, msg.addr);
        } else {
            info!("No packets received.");
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }
}

async fn cleanup_task(config: Config, agent_store: SharedAgentStore) -> Result<(), DiscoveryError> {
    let interval = config.agent_cleanup_interval;
    let duration = Duration::from_secs(interval);

    info!("Starting agent store cleanup task...");

    loop {
        tokio::time::sleep(duration).await;
        let mut store = agent_store.write().await;
        store.cleanup();
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
