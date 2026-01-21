use anyhow::Result;
use state_server::HttpServer;

use common::get_env_var;
use std::sync::Arc;
use tokio::task::JoinSet;
use tracing::info;
use udp_discovery::{Config, UdpDiscovery};

fn read_config() -> Config {
    Config::new(
        get_env_var("DISCOVERY_INTERVAL"),
        get_env_var("DISCOVERY_PORT"),
        get_env_var("AGENT_TTL"),
        get_env_var("AGENT_CLEANUP_INTERVAL"),
    )
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let config = read_config();
    let agent_id = get_env_var("AGENT_ID");
    let http_port = get_env_var("HTTP_PORT");

    info!("Starting discovery process...");

    let state_server = HttpServer::new(http_port);
    let discovery = Arc::new(UdpDiscovery::new(agent_id, config));
    let discovery_clone = discovery.clone();

    let mut tasks = JoinSet::<Result<()>>::new();

    tasks.spawn(async move {
        discovery.start().await;
        Ok(())
    });

    tasks.spawn(async move {
        state_server.run(discovery_clone).await?;
        Ok(())
    });

    while let Some(res) = tasks.join_next().await {
        match res {
            Ok(Ok(())) => {
                panic!("A background task exited unexpectedly");
            }
            Ok(Err(e)) => return Err(e),
            Err(e) => return Err(e.into()),
        }
    }

    Ok(())
}

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
