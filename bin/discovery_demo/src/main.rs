use agent_state::{AgentStore, Config};
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::task::JoinSet;
use tracing::info;
use udp_discovery::DiscoveryServer;

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::from_env();

    tracing_subscriber::fmt::init();

    info!("Starting discovery process...");

    let agent_store = Arc::new(RwLock::new(AgentStore::new(config.agent_ttl)));
    let discovery_server = DiscoveryServer::new(config, agent_store);

    let mut tasks = JoinSet::<Result<()>>::new();

    tasks.spawn(async move {
        discovery_server.start().await;
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
