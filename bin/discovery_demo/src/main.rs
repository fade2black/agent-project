use anyhow::{Result, anyhow};
use std::env;
use std::sync::Arc;
use tracing::info;
use udp_discovery::UdpDiscovery;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    info!("Starting discovery process...");

    let port: u16 = get_env_var("DISCOVERY_PORT")?;
    let interval_secs: u64 = get_env_var("DISCOVERY_INTERVAL")?;
    let agent_id: u32 = get_env_var("AGENT_ID")?;

    let service = Arc::new(UdpDiscovery::new(agent_id, interval_secs, port));
    service.start().await;

    // For graceful shutdown
    // let discovery_task = {
    //         let discovery = discovery.clone();
    //         tokio::spawn(async move {
    //             discovery.start().await;
    //         })
    //     };

    //     // Wait for Ctrl+C (or any other shutdown signal)
    //     signal::ctrl_c().await?;
    //     info!("Shutdown signal received, stopping discovery...");

    //
    //     discovery_task.abort();

    //     info!("Discovery service stopped.");
    //     Ok(())

    Ok(())
}

fn get_env_var<T: std::str::FromStr>(name: &str) -> Result<T> {
    let val = env::var(name).map_err(|_| anyhow!("Missing env var {}", name))?;
    val.parse::<T>()
        .map_err(|_| anyhow!("Failed to parse env var {}: {}", name, val))
}
