// Author: Sascha Zenglein <zenglein@gessler.de>
// Copyright (c) 2023 Gessler GmbH.

use super::smp::SmpTransport;
use super::smp_framing;
use crate::transport::error::Error;
use serialport::SerialPort;
use std::io::{BufRead, BufReader};
use std::time::Duration;

pub struct SerialTransport {
    serial_device: Box<dyn SerialPort>,
    buf: Vec<u8>,
}

impl SerialTransport {
    pub fn new(port: String, baud_rate: u32) -> Result<Self, Box<dyn std::error::Error>> {
        let serial = serialport::new(port, baud_rate).open_native()?;
        let buf = vec![0; 128];
        Ok(Self {
            serial_device: Box::new(serial),
            buf,
        })
    }

    pub fn recv_timeout(&mut self, timeout: Option<Duration>) -> Result<(), Error> {
        self.serial_device
            .set_timeout(timeout.unwrap_or(Duration::MAX))
            .map_err(|e| Error::Io(e.into()))
    }
}

impl SmpTransport for SerialTransport {
    fn send(&mut self, frame: Vec<u8>) -> Result<(), Error> {
        let mut encoder = smp_framing::SmpTransportEncoder::new(&frame);

        self.buf.resize(128, 0);
        while !encoder.is_complete() {
            let len = encoder
                .write_line(&mut self.buf)
                .expect("Buffer too small!");
            self.serial_device.write_all(&self.buf[0..len])?;
        }

        Ok(())
    }

    fn receive(&mut self) -> Result<Vec<u8>, Error> {
        let mut decoder = smp_framing::SmpTransportDecoder::new();
        let mut buf_reader = BufReader::new(&mut self.serial_device);
        while !decoder.is_complete() {
            self.buf.clear();
            let len = buf_reader.read_until(0xa, &mut self.buf)?;

            decoder.input_line(&self.buf[0..len])?;
        }

        let resp = decoder.into_frame_payload()?;

        Ok(resp)
    }
}
