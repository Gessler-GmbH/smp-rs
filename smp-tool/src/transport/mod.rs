// Author: Sascha Zenglein <zenglein@gessler.de>
// Copyright (c) 2023 Gessler GmbH.

use std::error::Error;
use std::time::Duration;

use mcumgr_smp::SMPFrame;

/// Serial transport implementation
pub mod serial;

/// UDP transport implementation
pub mod udp;

pub trait SMPTransport {
    /// send a single frame
    fn send(&mut self, frame: Vec<u8>) -> Result<(), Box<dyn Error>>;

    /// receive a single frame
    fn receive(&mut self) -> Result<Vec<u8>, Box<dyn Error>>;

    fn transceive(&mut self, frame: Vec<u8>) -> Result<Vec<u8>, Box<dyn Error>> {
        self.send(frame)?;
        self.receive()
    }

    /// set the timeout for a receive operation
    fn recv_timeout(&mut self, timeout: Option<Duration>) -> Result<(), Box<dyn Error>>;
}

pub struct CborSMPTransport {
    pub transport: Box<dyn SMPTransport>,
}

impl CborSMPTransport {
    pub fn send(&mut self, frame: Vec<u8>) -> Result<(), Box<dyn Error>> {
        self.transport.send(frame)
    }
    pub fn receive(&mut self) -> Result<Vec<u8>, Box<dyn Error>> {
        self.transport.receive()
    }

    pub fn transceive(&mut self, frame: Vec<u8>) -> Result<Vec<u8>, Box<dyn Error>> {
        self.transport.transceive(frame)
    }

    pub fn send_cbor<T: serde::Serialize>(
        &mut self,
        frame: SMPFrame<T>,
    ) -> Result<(), Box<dyn Error>> {
        let bytes = frame.encode_with_cbor();
        self.send(bytes)
    }
    pub fn receive_cbor<T: serde::de::DeserializeOwned>(
        &mut self,
    ) -> Result<SMPFrame<T>, Box<dyn Error>> {
        let bytes = self.receive()?;
        let frame = SMPFrame::<T>::decode_with_cbor(&bytes)?;
        Ok(frame)
    }

    pub fn transceive_cbor<Req: serde::Serialize, Resp: serde::de::DeserializeOwned>(
        &mut self,
        frame: SMPFrame<Req>,
    ) -> Result<SMPFrame<Resp>, Box<dyn Error>> {
        self.send_cbor(frame)?;
        self.receive_cbor()
    }

    /// set the timeout for a receive operation
    pub fn recv_timeout(&mut self, timeout: Option<Duration>) -> Result<(), Box<dyn Error>> {
        self.transport.recv_timeout(timeout)
    }
}
