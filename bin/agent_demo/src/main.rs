use agent_state::{AgentStore, TaskStore};
use anyhow::Result;
use state_server::StateServerContext;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::task::JoinSet;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    info!("Agent starting...");

    let task_store = Arc::new(RwLock::new(TaskStore::new()));
    let agent_store = Arc::new(RwLock::new(AgentStore::new()));

    let state = StateServerContext {
        task_store: task_store.clone(),
        agent_store: agent_store.clone(),
    };

    let mut tasks = JoinSet::<Result<()>>::new();

    tasks.spawn(async move {
        udp_discovery::start(agent_store).await;
        Ok(())
    });

    tasks.spawn(async move {
        control_server::start(task_store).await?;
        Ok(())
    });

    tasks.spawn(async move {
        state_server::start(state).await?;
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
