#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Io: {0}")]
    Io(#[from] std::io::Error),
    #[error("SMP: {0}")]
    Smp(#[from] crate::smp::SMPError),
    #[cfg(feature = "transport-serial")]
    #[error("SmpTransport: {0}")]
    SmpTransport(#[from] crate::smp_framing::SmpTransportError),
    #[cfg(feature = "transport-ble-async")]
    #[error("Bluetooth transport: {0}")]
    BLE(#[from] btleplug::Error),
}

pub type Result<T = (), E = Error> = core::result::Result<T, E>;