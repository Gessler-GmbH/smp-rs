// Author: Sascha Zenglein <zenglein@gessler.de>
// Copyright (c) 2023 Gessler GmbH.

//! A library that implements the SMP protocol.
//!
//! This library aims to be compatible with the Zepyhr SMP implementation.
//! For more information, see <https://docs.zephyrproject.org/3.1.0/services/device_mgmt/smp_protocol.html>
//!
//! Apart from [smp_framing], messages are encoded and decoded from and to raw byte buffers.
//! You must provide your own transport implementation.

/// Implementation of a general [SmpFrame] that can have any payload.
pub mod smp;

#[cfg(feature = "payload-cbor")]
pub mod application_management;
#[cfg(feature = "payload-cbor")]
pub mod os_management;
#[cfg(feature = "payload-cbor")]
pub mod shell_management;

/// Implementations over Serial, BLE and UDP transports
pub mod transport;

/// Support for the [SMP text console transport](https://github.com/apache/mynewt-mcumgr/blob/master/transport/smp-console.md)
#[cfg(feature = "transport-serial")]
pub mod smp_framing;

pub use smp::*;
