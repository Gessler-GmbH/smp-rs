// Author: Sascha Zenglein <zenglein@gessler.de>
// Copyright (c) 2023 Gessler GmbH.

/// Serial transport implementation
#[cfg(feature = "transport-serial")]
pub mod serial;
/// Support for the [SMP text console transport](https://github.com/apache/mynewt-mcumgr/blob/master/transport/smp-console.md)
#[cfg(feature = "transport-serial")]
pub mod smp_framing;

/// UDP transport implementation
#[cfg(any(feature = "transport-udp", feature = "transport-udp-async"))]
pub mod udp;

/// BLE transport implementation
#[cfg(feature = "transport-ble-async")]
pub mod ble;

pub mod error;

pub mod smp;
