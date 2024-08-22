// Author: Sascha Zenglein <zenglein@gessler.de>
// Copyright (c) 2023 Gessler GmbH.

use crate::transport::error::Error;
use crate::transport::smp::SmpTransport;
use std::io;
use std::net::{Ipv6Addr, SocketAddr, ToSocketAddrs, UdpSocket};
use std::time::Duration;

pub struct UdpTransport {
    socket: UdpSocket,
    buf: Vec<u8>,
}

impl UdpTransport {
    pub fn new<A: ToSocketAddrs>(
        target: A,
        recv_timeout: Option<Duration>,
    ) -> Result<Self, io::Error> {
        let socket = UdpSocket::bind(SocketAddr::new(Ipv6Addr::UNSPECIFIED.into(), 0))?;
        socket.connect(target)?;
        socket.set_read_timeout(recv_timeout)?;

        let buf = vec![0; 1500];

        Ok(Self { socket, buf })
    }
}

impl SmpTransport for UdpTransport {
    fn send(&mut self, frame: Vec<u8>) -> Result<(), Error> {
        self.socket.send(&frame)?;
        Ok(())
    }

    fn receive(&mut self) -> Result<Vec<u8>, Error> {
        let len = self.socket.recv(&mut self.buf)?;

        Ok(Vec::from(&self.buf[0..len]))
    }
}
