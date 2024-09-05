// Author: Sascha Zenglein <zenglein@gessler.de>
// Copyright (c) 2023 Gessler GmbH.

use crate::transport::error::Error;
use crate::transport::smp::SmpTransportAsync;
use async_trait::async_trait;
use std::io;
use std::net::{Ipv6Addr, SocketAddr};
use tokio::net::{ToSocketAddrs, UdpSocket};

pub struct UdpTransportAsync {
    socket: UdpSocket,
    buf: Vec<u8>,
}

impl UdpTransportAsync {
    pub async fn new<A: ToSocketAddrs>(target: A) -> Result<Self, io::Error> {
        let socket = UdpSocket::bind(SocketAddr::new(Ipv6Addr::UNSPECIFIED.into(), 0)).await?;
        socket.connect(target).await?;

        let buf = vec![0; 1500];

        Ok(Self { socket, buf })
    }
}

#[async_trait]
impl SmpTransportAsync for UdpTransportAsync {
    async fn send(&mut self, frame: Vec<u8>) -> Result<(), Error> {
        self.socket.send(&frame).await?;
        Ok(())
    }

    async fn receive(&mut self) -> Result<Vec<u8>, Error> {
        let len = self.socket.recv(&mut self.buf).await?;

        Ok(Vec::from(&self.buf[0..len]))
    }
}
