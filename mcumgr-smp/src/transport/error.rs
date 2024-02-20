#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Io: {0}")]
    Io(#[from] std::io::Error),
    #[error("SMP: {0}")]
    SMP(#[from] crate::SMPError),
    #[cfg(feature = "serial")]
    #[error("SMPTransport: {0}")]
    SMPTransport(#[from] crate::smp_framing::SMPTransportError),
    #[cfg(feature = "transport-serial")]
    #[error("Serial transport: {0}")]
    Serial(#[from] tokio_serial::Error),
}

pub type Result<T = (), E = Error> = core::result::Result<T, E>;
