#[cfg(feature = "async")]
pub mod smp_async;
#[cfg(all(feature = "payload-cbor", feature = "async"))]
pub use smp_async::cbor::CborSmpTransportAsync;
#[cfg(feature = "async")]
pub use smp_async::SmpTransportAsync;

pub mod smp_sync;
#[cfg(feature = "payload-cbor")]
pub use smp_sync::cbor::CborSmpTransport;
pub use smp_sync::SmpTransport;
