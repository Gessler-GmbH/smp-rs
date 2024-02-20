mod error;
pub use error::{Error, Result};

use crate::SMPFrame;

#[cfg(feature = "transport-serial")]
pub mod serial;

#[cfg(feature = "transport-udp")]
pub mod udp;

/// Async Transport layer trait.
pub trait AsyncSMPTransport {
    /// Send a single frame.
    fn send(&mut self, frame: &[u8]) -> impl core::future::Future<Output = Result> + Send;

    /// Receive a single frame.
    fn receive(&mut self) -> impl core::future::Future<Output = Result<Vec<u8>>> + Send;

    /// Set timeout for the transport.
    fn set_recv_timeout(&mut self, timeout: Option<std::time::Duration>) -> Result;
}

/// Async CBOR encoded frame helper.
pub struct AsyncCBORSMPTransport<T: AsyncSMPTransport> {
    /// Underlying transport.
    pub transport: T,
}

impl<T: AsyncSMPTransport> AsyncCBORSMPTransport<T> {
    /// Sends raw bytes.
    ///
    /// This calls [AsyncSMPTransport::send].
    #[inline]
    pub async fn send_bytes(&mut self, frame: &[u8]) -> Result {
        self.transport.send(frame).await
    }

    /// Receives raw bytes.
    ///
    /// This calls [AsyncSMPTransport::receive].
    #[inline]
    pub async fn receive_bytes(&mut self) -> Result<Vec<u8>> {
        self.transport.receive().await
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

    /// Set the receiving timeout for the transport.
    #[inline]
    pub fn set_recv_timeout(&mut self, timeout: Option<std::time::Duration>) -> Result {
        self.transport.set_recv_timeout(timeout)
    }
}
