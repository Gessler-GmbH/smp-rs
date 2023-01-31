// Author: Sascha Zenglein <zenglein@gessler.de>
// Copyright (c) 2023 Gessler GmbH.

use std::error::Error;
use std::io::{BufRead, BufReader};
use std::time::Duration;

use serialport::SerialPort;

use crate::transport::SMPTransport;
use mcumgr_smp::smp_framing;

pub struct SerialTransport {
    pub(crate) serial_device: Box<dyn SerialPort>,
    pub(crate) buf: Vec<u8>,
}

impl SMPTransport for SerialTransport {
    fn send(&mut self, frame: Vec<u8>) -> Result<(), Box<dyn Error>> {
        let mut encoder = smp_framing::SMPTransportEncoder::new(&frame);

        self.buf.resize(128, 0);
        while !encoder.is_complete() {
            let len = encoder
                .write_line(&mut self.buf)
                .expect("Buffer too small!");
            self.serial_device.write_all(&self.buf[0..len])?;
        }

        Ok(())
    }

    fn receive(&mut self) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut decoder = smp_framing::SMPTransportDecoder::new();
        let mut buf_reader = BufReader::new(&mut self.serial_device);
        while !decoder.is_complete() {
            self.buf.clear();
            let len = buf_reader.read_until(0xa, &mut self.buf)?;

            decoder.input_line(&self.buf[0..len])?;
        }

        let resp = decoder.into_frame_payload()?;

        Ok(resp)
    }

    fn recv_timeout(&mut self, timeout: Option<Duration>) -> Result<(), Box<dyn Error>> {
        let timeout = timeout.unwrap_or(Duration::MAX);

        self.serial_device.set_timeout(timeout)?;

        Ok(())
    }
}
