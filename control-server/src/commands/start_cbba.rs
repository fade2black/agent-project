use crate::listener::{CommandType, ControlCommand, ControlCommandError};
//use agent_state::DefaultTask;
use bytes::Bytes;
//use common::RmpSerializable;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct StartCbba;

impl TryInto<ControlCommand> for StartCbba {
    type Error = ControlCommandError;

    fn try_into(self) -> Result<ControlCommand, Self::Error> {
        Ok(ControlCommand::new(CommandType::StartCbba, Bytes::new()))
    }
}

impl TryFrom<ControlCommand> for StartCbba {
    type Error = ControlCommandError;

    fn try_from(cmd: ControlCommand) -> Result<Self, Self::Error> {
        if cmd.tp != CommandType::StartCbba {
            return Err(ControlCommandError::WrongType);
        }

        Ok(StartCbba)
    }
}
