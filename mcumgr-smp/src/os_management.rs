// Author: Sascha Zenglein <zenglein@gessler.de>
// Copyright (c) 2023 Gessler GmbH.
use crate::{Group, SmpFrame};

use crate::OpCode::{ReadRequest, WriteRequest};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct EchoRequest {
    pub d: String,
}

pub fn echo(sequence: u8, msg: String) -> SmpFrame<EchoRequest> {
    let payload = EchoRequest { d: msg };
    SmpFrame::new(WriteRequest, sequence, Group::Default, 0, payload)
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum EchoResult {
    Ok { r: String },
    Err { rc: i32 },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetInfoRequest {
    pub format: String,
}

pub fn get_info(sequence: u8, format: String) -> SmpFrame<GetInfoRequest> {
    let request = GetInfoRequest { format };

    SmpFrame::new(ReadRequest, sequence, Group::Default, 7, request)
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum ResetResult {
    Ok {},
    Err { rc: i32 },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ResetRequest {
    pub force: u8,
}

pub fn reset(sequence: u8, force: bool) -> SmpFrame<ResetRequest> {
    let payload = ResetRequest { force: force as u8 };

    SmpFrame::new(WriteRequest, sequence, Group::Default, 5, payload)
}
