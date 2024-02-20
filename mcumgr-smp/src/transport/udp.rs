//! UDP transport layer implementation.

use std::{io, net::Ipv6Addr};

use super::{AsyncSMPTransport, Result};

pub struct AsyncUDPTransport {
    socket: tokio::net::UdpSocket,
    buf: Vec<u8>,
    timeout: Option<std::time::Duration>,
}

impl AsyncUDPTransport {
    /// Create a new [`AsyncUDPTransport`] with default mtu of 1500.
    ///
    /// See [`new_with_mtu`] for custom mtu value.
    ///
    /// [`new_with_mtu`]: Self::new_with_mtu
    #[inline]
    pub async fn new<A: tokio::net::ToSocketAddrs>(target: A) -> Result<Self> {
        Self::new_with_mtu(target, 1500).await
    }

    /// Create a new [`AsyncUDPTransport`] with a given MTU for the internal buffer.
    pub async fn new_with_mtu<A: tokio::net::ToSocketAddrs>(target: A, mtu: usize) -> Result<Self> {
        let socket = tokio::net::UdpSocket::bind((Ipv6Addr::UNSPECIFIED, 0)).await?;
        socket.connect(target).await?;

        let mut buf = Vec::with_capacity(mtu);
        buf.resize(mtu, 0);

        Ok(Self {
            socket,
            buf,
            timeout: None,
        })
    }
}

impl AsyncSMPTransport for AsyncUDPTransport {
    #[inline]
    async fn send(&mut self, frame: &[u8]) -> Result {
        self.socket.send(&frame).await?;
        Ok(())
    }

    async fn receive(&mut self) -> Result<Vec<u8>> {
        let timeout = self.timeout.clone();
        let future = self.socket.recv(&mut self.buf);
        let len = if let Some(timeout) = timeout {
            tokio::time::timeout(timeout, future).await.map_err(|_| {
                super::Error::Io(io::Error::new(
                    io::ErrorKind::TimedOut,
                    "Time out while receiving udp frame",
                ))
            })?
        } else {
            future.await
        }?;
        Ok(Vec::from(&self.buf[..len]))
    }

    #[inline]
    fn set_recv_timeout(&mut self, timeout: Option<std::time::Duration>) -> Result {
        self.timeout = timeout;
        Ok(())
    }
}
