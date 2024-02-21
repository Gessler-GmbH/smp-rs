//! UDP transport layer implementation.

use core::future::Future;
use core::pin::Pin;
use std::{io, net::Ipv6Addr};

use super::{AsyncSMPTransport, Result};

pub struct AsyncUDPTransport {
    socket: tokio::net::UdpSocket,
    buf: Vec<u8>,
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

        Ok(Self { socket, buf })
    }
}

impl AsyncSMPTransport for AsyncUDPTransport {
    fn poll_send(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
        frame: &[u8],
    ) -> core::task::Poll<Result> {
        let me = self.get_mut();
        core::task::ready!(Pin::new(&mut me.socket).poll_send(cx, &frame))?;
        core::task::Poll::Ready(Ok(()))
    }

    fn poll_receive(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Result<Vec<u8>>> {
        let me = self.get_mut();
        let len = core::task::ready!(core::pin::pin!(me.socket.recv(&mut me.buf)).poll(cx))?;
        core::task::Poll::Ready(Ok(Vec::from(&me.buf[..len])))
    }
}
