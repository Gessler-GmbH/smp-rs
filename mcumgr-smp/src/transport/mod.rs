mod error;

use core::ops::DerefMut;
use core::{
    pin::Pin,
    task::{Context, Poll},
};

pub use error::{Error, Result};

use crate::SMPFrame;

mod util;

#[cfg(feature = "transport-serial")]
pub mod serial;

#[cfg(feature = "transport-udp")]
pub mod udp;

#[cfg(feature = "transport-ble")]
pub mod ble;

/// Async Transport layer trait.
pub trait AsyncSMPTransport {
    /// Send a single frame.
    fn poll_send(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
        frame: &[u8],
    ) -> core::task::Poll<Result>;

    /// Receive a single frame.
    fn poll_receive(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Result<Vec<u8>>>;
}

impl<P> AsyncSMPTransport for core::pin::Pin<P>
where
    P: DerefMut + Unpin,
    P::Target: AsyncSMPTransport + Unpin,
{
    fn poll_send(self: Pin<&mut Self>, cx: &mut Context<'_>, frame: &[u8]) -> Poll<Result> {
        self.get_mut().as_mut().poll_send(cx, frame)
    }

    fn poll_receive(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<Vec<u8>>> {
        self.get_mut().as_mut().poll_receive(cx)
    }
}

macro_rules! deref_async_transport {
    () => {
        fn poll_send(mut self: Pin<&mut Self>, cx: &mut Context<'_>, frame: &[u8]) -> Poll<Result> {
            Pin::new(&mut **self).poll_send(cx, frame)
        }

        fn poll_receive(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<Vec<u8>>> {
            Pin::new(&mut **self).poll_receive(cx)
        }
    };
}

impl<T: ?Sized + AsyncSMPTransport + Unpin> AsyncSMPTransport for Box<T> {
    deref_async_transport!();
}

impl<T: ?Sized + AsyncSMPTransport + Unpin> AsyncSMPTransport for &mut T {
    deref_async_transport!();
}

/// Async CBOR encoded frame helper.
pub struct AsyncCBORSMPTransport<T: AsyncSMPTransport = Box<dyn AsyncSMPTransport + Unpin>> {
    /// Underlying transport.
    pub transport: T,
}

impl<T: AsyncSMPTransport> AsyncCBORSMPTransport<T> {
    /// Create a new instance of `Self`.
    pub fn new(transport: T) -> Self {
        Self { transport }
    }
}

impl<T: AsyncSMPTransport + Unpin> AsyncCBORSMPTransport<T> {
    /// Sends raw bytes.
    ///
    /// This calls [AsyncSMPTransport::send].
    #[inline]
    pub fn send_bytes<'a>(&'a mut self, frame: &'a [u8]) -> util::Send<'a, T> {
        util::send(&mut self.transport, frame)
    }

    /// Receives raw bytes.
    ///
    /// This calls [AsyncSMPTransport::receive].
    #[inline]
    pub fn receive_bytes<'a>(&'a mut self) -> util::Receive<'a, T> {
        util::receive(&mut self.transport)
    }

    /// Sends a CBOR frame
    pub async fn send<F: serde::Serialize>(&mut self, frame: SMPFrame<F>) -> Result {
        let bytes = frame.encode_with_cbor();
        self.send_bytes(&bytes).await
    }

    /// Receive a CBOR frame.
    pub async fn receive<F: serde::de::DeserializeOwned>(&mut self) -> Result<SMPFrame<F>> {
        let bytes = self.receive_bytes().await?;
        let frame = SMPFrame::<F>::decode_with_cbor(&bytes)?;
        Ok(frame)
    }

    /// Send and receive a CBOR frame.
    ///
    /// Calls [`send`] and then [`receive`].
    ///
    /// [`send`]: Self::send
    /// [`receive`]: Self::receive
    pub async fn transceive<TX: serde::Serialize, RX: serde::de::DeserializeOwned>(
        &mut self,
        frame: SMPFrame<TX>,
    ) -> Result<SMPFrame<RX>> {
        self.send(frame).await?;
        self.receive().await
    }
}
