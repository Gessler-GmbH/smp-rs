// Author: Sascha Zenglein <zenglein@gessler.de>
// Copyright (c) 2023 Gessler GmbH.

use std::error::Error;
use std::io;
use std::net::{Ipv6Addr, SocketAddr, ToSocketAddrs, UdpSocket};
use std::time::Duration;

use crate::transport::SMPTransport;

pub struct UdpTransport {
    socket: UdpSocket,
    buf: Vec<u8>,
}

impl UdpTransport {
    pub fn new<A: ToSocketAddrs>(target: A) -> Result<Self, io::Error> {
        let socket = UdpSocket::bind(SocketAddr::new(Ipv6Addr::UNSPECIFIED.into(), 0))?;
        socket.connect(target)?;

        let buf = vec![0; 1500];

        Ok(Self { socket, buf })
    }
}

impl SMPTransport for UdpTransport {
    fn send(&mut self, frame: Vec<u8>) -> Result<(), Box<dyn Error>> {
        self.socket.send(&frame)?;
        Ok(())
    }

    fn receive(&mut self) -> Result<Vec<u8>, Box<dyn Error>> {
        let len = self.socket.recv(&mut self.buf)?;

        Ok(Vec::from(&self.buf[0..len]))
    }

    fn recv_timeout(&mut self, timeout: Option<Duration>) -> Result<(), Box<dyn Error>> {
        self.socket.set_read_timeout(timeout)?;
        Ok(())
    }
}
