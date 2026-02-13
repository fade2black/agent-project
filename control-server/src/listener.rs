use crate::commands::DistributeTasks;
use agent_state::Config;
use agent_state::ControlState;
use agent_state::SharedAgentState;
use bytes::Bytes;
use cbba::CbbaRunner;
use cbba::cbba_runner;
use common::time::now;
use common::{RmpSerializable, SerializationError};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;
use tracing::{error, info, warn};
use transport::Transport;
use udp_transport::UdpTransport;

const MAX_COMMAND_SIZE: usize = 1024;

#[derive(Debug, Error)]
pub enum ControlCommandError {
    #[error("Transport error: {0}")]
    Transport(#[from] udp_transport::TransportError),
    #[error("Cbba error: {0}")]
    Cbba(#[from] cbba_runner::CbbaError),
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

pub struct ControlServer {
    config: Config,
    agent_state: Arc<SharedAgentState>,
}

impl ControlServer {
    pub fn new(config: Config, agent_state: Arc<SharedAgentState>) -> Self {
        ControlServer {
            agent_state,
            config,
        }
    }

    pub async fn start(&self) -> Result<(), ControlCommandError> {
        let port = self.config.command_control_port;

        let mut bytes = [0u8; MAX_COMMAND_SIZE];
        let mut receiver = UdpTransport::new_receiver(port).await?;

        info!("Starting control command listener...");

        loop {
            let size = match receiver.recv(&mut bytes).await {
                Ok(size) => size,
                Err(e) => {
                    error!("Error receiving control command: {}", e);
                    continue;
                }
            };

            let Some(cmd) = parse_control_command(&bytes[..size]) else {
                continue;
            };

            match cmd.tp {
                CommandType::StartCbba => {
                    info!("Received start-cbba command.");
                    self.start_cbba().await?;
                }
                CommandType::DistributeTasks => {
                    info!("Received task distribution command.");
                    self.distribute_tasks(cmd).await?;
                }
            }
        }
    }

    async fn distribute_tasks(&self, cmd: ControlCommand) -> Result<(), ControlCommandError> {
        if !self.try_enter_state(ControlState::RunningDistTasks).await {
            warn!("Cannot distribute tasks: agent busy.");
            return Ok(());
        }

        let control_state = self.agent_state.control_state.clone();
        let cmd = DistributeTasks::try_from(cmd)?;

        let mut task_store = self.agent_state.task_store.write().await;
        task_store.clear();
        task_store.insert_tasks(cmd.tasks);
        info!("New tasks added.");

        *control_state.write().await = ControlState::Idle;

        Ok(())
    }

    async fn start_cbba(&self) -> Result<(), ControlCommandError> {
        if !self.try_enter_state(ControlState::RunningCBBA).await {
            warn!("Cannot start CBBA: agent busy.");
            return Ok(());
        }

        let shared_bundle = self.agent_state.bundle.clone();
        let shared_winners = self.agent_state.winners.clone();
        let control_state = self.agent_state.control_state.clone();
        let tasks = { self.agent_state.task_store.read().await.get_tasks() };

        let cbba_runner = CbbaRunner::new(self.config, shared_bundle, shared_winners, tasks);

        let _ = tokio::spawn(async move {
            if let Err(e) = cbba_runner.start().await {
                error!("CBBA failed: {}", e);
            }

            *control_state.write().await = ControlState::Idle;
        });

        Ok(())
    }

    // For just three states the following if-else is enough.
    // However, for more states, another approach is needed.
    async fn try_enter_state(&self, new_state: ControlState) -> bool {
        let mut state = self.agent_state.control_state.write().await;
        if *state == ControlState::Idle {
            *state = new_state;
            true
        } else {
            false
        }
    }
}

fn parse_control_command(bytes: &[u8]) -> Option<ControlCommand> {
    match ControlCommand::from_bytes(bytes) {
        Ok(cmd) => Some(cmd),
        Err(_) => {
            warn!("Dropping invalid control command.");
            None
        }
    }
}
