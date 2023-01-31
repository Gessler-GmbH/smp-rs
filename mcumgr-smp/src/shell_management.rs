// Author: Sascha Zenglein <zenglein@gessler.de>
// Copyright (c) 2023 Gessler GmbH.
use crate::{Group, SMPFrame};

use crate::OpCode::WriteRequest;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ShellCommand {
    /// argv containing cmd + arg, arg, ...
    pub argv: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum ShellResult {
    Ok { o: String, ret: i32 },
    Err { rc: i32 },
}

impl ShellResult {
    pub fn into_result(self) -> Result<(String, i32), i32> {
        match self {
            ShellResult::Ok { o, ret } => Ok((o, ret)),
            ShellResult::Err { rc } => Err(rc),
        }
    }
}

pub fn shell_command(sequence: u8, command_args: Vec<String>) -> SMPFrame<ShellCommand> {
    let payload = ShellCommand { argv: command_args };

    SMPFrame::new(WriteRequest, sequence, Group::ShellManagement, 0, payload)
}
