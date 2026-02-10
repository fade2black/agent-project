use agent_state::{Config, SharedAgentState};
use anyhow::Result;
use control_server::ControlServer;
use state_server::StateServer;
use std::sync::Arc;
use tokio::task::JoinSet;
use tracing::info;
use udp_discovery::DiscoveryServer;

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::from_env();
    let agent_state = Arc::new(SharedAgentState::new(config.agent_ttl));

    let agent_store = agent_state.agent_store.clone();
    let discovery_server = DiscoveryServer::new(config, agent_store);
    let control_server = ControlServer::new(config, agent_state.clone());
    let state_server = StateServer::new(config, agent_state.clone());

    tracing_subscriber::fmt::init();
    info!("Agent starting...");

    let mut tasks = JoinSet::<Result<()>>::new();

    tasks.spawn(async move {
        discovery_server.start().await;
        Ok(())
    });

    tasks.spawn(async move {
        control_server.start().await?;
        Ok(())
    });

    tasks.spawn(async move {
        state_server.start().await?;
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
