use common::ByteSerializable;
use if_addrs::{IfAddr, get_if_addrs};
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use std::sync::Arc;
use thiserror::Error;
use tokio::task::JoinSet;
use tokio::time::Duration;
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
    transport_addr: Option<IpAddr>,
}

impl HeartbeatMessage {
    pub fn new(agent_id: u32, transport_addr: Option<IpAddr>) -> Self {
        Self {
            agent_id,
            transport_addr,
        }
    }
}

pub struct UdpDiscovery {
    agent_id: u32,
    interval_sec: u64,
    port: u16,
}

impl UdpDiscovery {
    pub fn new(agent_id: u32, interval_sec: u64, port: u16) -> Self {
        Self {
            agent_id,
            interval_sec,
            port,
        }
    }

    pub async fn start(self: Arc<Self>) {
        let mut tasks = JoinSet::new();

        tasks.spawn(send_heartbeat_task(self.clone()));
        tasks.spawn(recv_heartbeat_task(self.clone()));

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
    let ips = retrieve_usable_ips()?;
    let duration = Duration::from_secs(discovery.interval_sec);
    let mut sender = UdpTransport::new_sender(discovery.port).await?;

    info!("Starting sending heartbeat task...");
    info!(
        "Port {}, Interval: {} seconds.",
        discovery.port, discovery.interval_sec
    );

    loop {
        let msg = HeartbeatMessage::new(discovery.agent_id, Some(ips[0]));
        let bytes = msg.to_bytes()?;

        // TODO: specific error handling for send() could be beneficial for debuging
        sender.send(&bytes).await?;
        info!("Heartbeat message sent.");

        tokio::time::sleep(duration).await;
    }
}

async fn recv_heartbeat_task(discovery: Arc<UdpDiscovery>) -> Result<(), DiscoveryError> {
    let mut receiver = UdpTransport::new_receiver(discovery.port).await?;
    let mut buffer = [0u8; MAX_HEARTBEAT_SIZE];

    info!("Starting receiving heartbeat task...");
    loop {
        match receiver.recv(&mut buffer).await? {
            Some(size) => {
                let msg = HeartbeatMessage::from_bytes(&buffer[..size])?;

                if msg.agent_id != discovery.agent_id {
                    info!("Received heartbeat {} bytes from {}", size, msg.agent_id);

                    // add to agent store
                }
            }
            None => {
                info!("No packets received.");
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
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
