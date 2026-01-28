use anyhow::{Result, anyhow};
use std::env;
use tokio::time::{Duration, sleep};
use tracing::info;
use transport::Transport;
use udp_transport::UdpTransport;

const BUFFER_SIZE: usize = 1024;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    info!("Starting...");

    let broadcast_port: u16 = get_env_var("BROADCAST_PORT")?;
    let broadcast_interval: u64 = get_env_var("BROADCAST_INTERVAL")?;

    let mut sender = UdpTransport::new_sender(broadcast_port).await?;
    let mut receiver = UdpTransport::new_receiver(broadcast_port).await?;

    tokio::spawn(async move {
        info!("Starting receiver task...");
        let mut buf = vec![0u8; BUFFER_SIZE];
        loop {
            if let Ok(len) = receiver.recv(&mut buf).await {
                info!("Received: {:?}", &buf[..len]);
            }
        }
    });

    let mut counter: usize = 0;
    info!("Starting sender loop...");
    loop {
        let msg = format!("Hello {}", counter);
        sender.send(msg.as_bytes()).await?;
        info!("Sent: {}", msg);

        sleep(Duration::from_secs(broadcast_interval)).await;
        counter = counter.wrapping_add(1);
    }
}

fn get_env_var<T: std::str::FromStr>(name: &str) -> Result<T> {
    let val = env::var(name).map_err(|_| anyhow!("Missing env var {}", name))?;
    val.parse::<T>()
        .map_err(|_| anyhow!("Failed to parse env var {}: {}", name, val))
}
