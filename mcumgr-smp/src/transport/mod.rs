mod error;
pub use error::{Error, Result};

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
    fn set_timeout(&mut self, timeout: Option<std::time::Duration>) -> Result;
}
