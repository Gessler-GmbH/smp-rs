// Author: Sascha Zenglein <zenglein@gessler.de>
// Copyright (c) 2023 Gessler GmbH.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum SMPError {
    #[error("payload decoding error: {0}")]
    PayloadDecodingError(#[from] Box<dyn std::error::Error>),
    #[error("smp frame decoding error")]
    InvalidFrame,
}

#[derive(Debug, Clone, Copy)]
pub enum OpCode {
    ReadRequest = 0,
    ReadResponse = 1,
    WriteRequest = 2,
    WriteResponse = 3,
}

impl From<u8> for OpCode {
    fn from(num: u8) -> Self {
        match num {
            0 => OpCode::ReadRequest,
            1 => OpCode::ReadResponse,
            2 => OpCode::WriteRequest,
            3 => OpCode::WriteResponse,
            _ => panic!("unknown opcode"),
        }
    }
}

impl From<OpCode> for u8 {
    fn from(op: OpCode) -> Self {
        match op {
            OpCode::ReadRequest => 0,
            OpCode::ReadResponse => 1,
            OpCode::WriteRequest => 2,
            OpCode::WriteResponse => 3,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Group {
    Default,
    ApplicationManagement,
    Statistics,
    FileManagement,
    ShellManagement,
    ZephyrCommand,
    Custom(u16),
}

impl From<u16> for Group {
    fn from(num: u16) -> Self {
        match num {
            0 => Self::Default,
            1 => Self::ApplicationManagement,
            2 => Self::Statistics,
            8 => Self::FileManagement,
            9 => Self::ShellManagement,
            63 => Self::ZephyrCommand,
            num => Self::Custom(num),
        }
    }
}

impl From<Group> for u16 {
    fn from(g: Group) -> Self {
        match g {
            Group::Default => 0,
            Group::ApplicationManagement => 1,
            Group::Statistics => 2,
            Group::FileManagement => 8,
            Group::ShellManagement => 9,
            Group::ZephyrCommand => 63,
            Group::Custom(num) => num,
        }
    }
}

pub enum ReturnCode {
    Ok = 0,
    Unknown = 1,
    OutOfMemory = 2,
    // ...
    UserDefined = 256,
}

/// Definitition of a single SMP message.  
/// SMP Requests and Responses always have this format.
#[derive(Debug, Clone)]
pub struct SMPFrame<T> {
    pub operation: OpCode,
    pub flags: u8,
    pub group: Group,
    pub sequence: u8,
    pub command: u8,
    pub data: T,
}

impl<T> SMPFrame<T> {
    ///  Create new with default flags
    pub fn new(operation: OpCode, sequence: u8, group: Group, command: u8, payload: T) -> Self {
        Self {
            operation,
            flags: 0,
            group,
            sequence,
            command,
            data: payload,
        }
    }
}

impl<T> SMPFrame<T> {
    /// Encode the frame to bytes using the given encode_payload handler.  
    /// For the common CBOR serialisation, see [SMPFrame::encode_with_cbor]
    pub fn encode<E, R: AsRef<[u8]>>(
        &self,
        encode_payload: impl FnOnce(&T) -> Result<R, E>,
    ) -> Result<Vec<u8>, E> {
        let mut buf: Vec<u8> = Vec::with_capacity(12);

        let encoded = encode_payload(&self.data)?;
        let data: &[u8] = encoded.as_ref();

        buf.push(self.operation.into());
        buf.push(self.flags);
        buf.extend_from_slice(&(data.len() as u16).to_be_bytes());
        let group: u16 = self.group.into();
        buf.extend_from_slice(&group.to_be_bytes());
        buf.push(self.sequence);
        buf.push(self.command);

        buf.extend(data.iter());

        Ok(buf)
    }

    /// Decode the frame from bytes using the given decode_payload handler.  
    /// For the common CBOR serialisation, see [SMPFrame::decode_with_cbor]
    pub fn decode(
        buf: &[u8],
        decode_payload: impl FnOnce(&[u8]) -> Result<T, Box<dyn std::error::Error>>,
    ) -> Result<SMPFrame<T>, SMPError> {
        if buf.len() < 8 {
            return Err(SMPError::InvalidFrame);
        }

        let operation = OpCode::from(buf[0] & 0x07);
        let group = Group::from(u16::from_be_bytes([buf[4], buf[5]]));
        let data_len = u16::from_be_bytes([buf[2], buf[3]]);
        let _flags = buf[1];
        let sequence = buf[6];
        let command = buf[7];

        if buf.len() < (8 + data_len) as usize {
            return Err(SMPError::InvalidFrame);
        }

        let data_buf = &buf[8..(8 + data_len as usize)];
        let data = decode_payload(data_buf)?;

        Ok(SMPFrame::new(operation, sequence, group, command, data))
    }
}

#[cfg(feature = "payload-cbor")]
impl<T: serde::Serialize> SMPFrame<T> {
    /// Encode the frame to bytes using CBOR serialization.  
    /// This method requires Serde
    pub fn encode_with_cbor(&self) -> Vec<u8> {
        // buf cannot run out of space because it can allocate
        self.encode(|data| {
            let mut buf = Vec::new();
            match ciborium::ser::into_writer(data, &mut buf) {
                Ok(_) => Ok(buf),
                Err(e) => Err(e),
            }
        })
        .unwrap()
    }
}

#[cfg(feature = "payload-cbor")]
impl<T: serde::de::DeserializeOwned> SMPFrame<T> {
    /// Decode the frame to bytes using CBOR deserialization.  
    /// This method requires Serde
    pub fn decode_with_cbor(buf: &[u8]) -> Result<SMPFrame<T>, SMPError> {
        Self::decode(buf, |buf| {
            let x: T = ciborium::de::from_reader(buf)?;
            Ok(x)
        })
    }
}
