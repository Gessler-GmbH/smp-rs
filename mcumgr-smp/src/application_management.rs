// Author: Sascha Zenglein <zenglein@gessler.de>
// Copyright (c) 2023 Gessler GmbH.

use crate::{Group, OpCode, SmpFrame};

use serde::{Deserialize, Serialize};

pub enum ApplicationManagementCommand {
    State,
    Upload,
    Erase,
    Unknown(u8),
}

impl From<ApplicationManagementCommand> for u8 {
    fn from(cmd: ApplicationManagementCommand) -> Self {
        match cmd {
            ApplicationManagementCommand::State => 0,
            ApplicationManagementCommand::Upload => 1,
            ApplicationManagementCommand::Erase => 5,
            ApplicationManagementCommand::Unknown(n) => n,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum GetImageStateResult {
    Ok(GetImageStatePayload),
    Err(GetImageStateError),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetImageStatePayload {
    pub images: Vec<ImageState>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub split_status: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetImageStateError {
    pub rc: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ImageState {
    pub image: Option<i32>,
    pub slot: i32,
    pub version: String,
    #[serde(with = "serde_bytes")]
    pub hash: Vec<u8>,
    #[serde(default)]
    pub bootable: bool,
    #[serde(default)]
    pub pending: bool,
    #[serde(default)]
    pub confirmed: bool,
    #[serde(default)]
    pub active: bool,
    #[serde(default)]
    pub permanent: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetStatePayload {}

pub fn get_state(sequence: u8) -> SmpFrame<GetStatePayload> {
    SmpFrame {
        operation: OpCode::ReadRequest,
        flags: 0,
        group: Group::ApplicationManagement,
        sequence,
        command: ApplicationManagementCommand::State.into(),
        data: GetStatePayload {},
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SetStatePayload {
    #[serde(with = "serde_bytes")]
    pub hash: Vec<u8>,
    pub confirm: bool,
}

pub fn set_state(hash: Vec<u8>, confirm: bool, sequence: u8) -> SmpFrame<SetStatePayload> {
    let data = SetStatePayload { hash, confirm };

    SmpFrame {
        operation: OpCode::WriteRequest,
        flags: 0,
        group: Group::ApplicationManagement,
        sequence,
        command: ApplicationManagementCommand::State.into(),
        data,
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ImageChunk<'d, 's> {
    #[serde(with = "serde_bytes")]
    pub data: &'d [u8],
    pub off: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub len: Option<usize>,
    #[serde(with = "serde_bytes")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha: Option<&'s [u8]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub upgrade: Option<bool>,
}

pub struct ImageWriter<'s> {
    pub image: Option<u8>,
    pub hash: Option<&'s [u8]>,
    pub offset: usize,
    pub len: usize,
    pub sequence: u8,
    pub upgrade: bool,
}

impl ImageWriter<'_> {
    pub fn new(image: Option<u8>, len: usize, hash: Option<&[u8]>, upgrade: bool) -> ImageWriter {
        ImageWriter {
            image,
            hash,
            offset: 0,
            len,
            sequence: 0,
            upgrade,
        }
    }

    pub fn write_chunk<'d>(&mut self, data: &'d [u8]) -> SmpFrame<ImageChunk<'d, '_>> {
        let data_len = data.len();

        let mut chunk_data = ImageChunk {
            data,
            off: self.offset,
            image: None,
            len: None,
            sha: None,
            upgrade: None,
        };

        if self.offset == 0 {
            chunk_data.len = Some(self.len);

            if let Some(image) = self.image {
                chunk_data.image = Some(image);
            }

            if let Some(hash) = self.hash {
                chunk_data.sha = Some(hash);
            }

            if self.upgrade {
                chunk_data.upgrade = Some(true);
            }
        }

        self.offset += data_len;

        (self.sequence, _) = self.sequence.overflowing_add(1);

        SmpFrame::new(
            OpCode::WriteRequest,
            self.sequence,
            Group::ApplicationManagement,
            ApplicationManagementCommand::Upload.into(),
            chunk_data,
        )
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum WriteImageChunkResult {
    Ok(WriteImageChunkPayload),
    Err(WriteImageChunkError),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WriteImageChunkPayload {
    pub off: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WriteImageChunkError {
    pub rc: i32,
}
