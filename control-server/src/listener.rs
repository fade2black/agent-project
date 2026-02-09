use crate::commands::DistributeTasks;
use agent_state::Config;
use agent_state::TaskStore;
use bytes::Bytes;
use common::time::now;
use common::{RmpSerializable, SerializationError};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;
use tracing::{error, info, warn};
use transport::Transport;
use udp_transport::UdpTransport;

const MAX_COMMAND_SIZE: usize = 1024;

#[derive(Debug, Error)]
pub enum ControlCommandError {
    #[error("Transport error: {0}")]
    Transport(#[from] udp_transport::TransportError),
    #[error("Serialization error: {0}")]
    Serialization(#[from] SerializationError),
    #[error("Wrong control command type")]
    WrongType,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum CommandType {
    StartCbba,
    DistributeTasks,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ControlCommand {
    pub tp: CommandType,
    pub bytes: Bytes,
    pub ts: u64,
}

impl ControlCommand {
    pub fn new(tp: CommandType, bytes: Bytes) -> Self {
        ControlCommand {
            tp,
            bytes,
            ts: now(),
        }
    }
}

pub async fn start(task_store: Arc<RwLock<TaskStore>>) -> Result<(), ControlCommandError> {
    let config = Config::new();
    let port = config.command_control_port;

    let mut receiver = UdpTransport::new_receiver(port).await?;
    let mut bytes = [0u8; MAX_COMMAND_SIZE];

    info!("Starting control command listener...");

    loop {
        match receiver.recv(&mut bytes).await {
            Ok(size) => {
                let Ok(cmd) = ControlCommand::from_bytes(&bytes[..size]) else {
                    warn!("Dropping invalid control command.");
                    continue;
                };

                info!("Received control command ({:?}).", cmd);
                process_command(cmd, task_store.clone()).await?;
            }
            Err(e) => {
                error!("Error receiving control command: {}", e);
                continue;
            }
        }
    }
}

async fn process_command(
    cmd: ControlCommand,
    task_store: Arc<RwLock<TaskStore>>,
) -> Result<(), ControlCommandError> {
    match cmd.tp {
        CommandType::StartCbba => {
            info!("Received start-cbba command.");
            //run_cbba_phase(state.clone()).await?;
        }
        CommandType::DistributeTasks => {
            info!("Received task distribution command.");
            let cmd = DistributeTasks::try_from(cmd)?;

            let mut task_store = task_store.write().await;
            task_store.clear();
            task_store.insert_tasks(cmd.tasks);
            info!("New tasks added.");
        }
    }
    Ok(())
}
