use crate::listener::{CommandType, ControlCommand, ControlCommandError};
use agent_state::Task;
use common::RmpSerializable;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct DistributeTasks {
    pub tasks: Vec<Task>,
}

impl DistributeTasks {
    pub fn new(tasks: Vec<Task>) -> Self {
        DistributeTasks { tasks }
    }
}

impl TryInto<ControlCommand> for DistributeTasks {
    type Error = ControlCommandError;

    fn try_into(self) -> Result<ControlCommand, Self::Error> {
        let bytes = self.tasks.to_bytes()?;
        Ok(ControlCommand::new(CommandType::DistributeTasks, bytes))
    }
}

impl TryFrom<ControlCommand> for DistributeTasks {
    type Error = ControlCommandError;

    fn try_from(cmd: ControlCommand) -> Result<Self, Self::Error> {
        if cmd.tp != CommandType::DistributeTasks {
            return Err(ControlCommandError::WrongType);
        }

        let tasks = Vec::from_bytes(&cmd.bytes)?;

        Ok(DistributeTasks { tasks })
    }
}
