// Author: Sascha Zenglein <zenglein@gessler.de>
// Copyright (c) 2024 Gessler GmbH.

//! A library that implements the SMP protocol.
//!
//! This library aims to be compatible with the Zepyhr SMP implementation.
//! For more information, see <https://docs.zephyrproject.org/3.1.0/services/device_mgmt/smp_protocol.html>
//!
//! # Transport
//! A transport implementation is provided for a number of different transports. Both
//! sync and async versions exist, but due to implementation effort and available crates
//! this varies for each transport type.  
//! See the [transport] module for more information.
//!
//! #### Bring your own transport
//! [SmpFrame] is implemented in such a way that it uses raw bytes (i.e. [Vec]) to encode or decode
//! messages. You can handle this conversion yourself and send these bytes over any channel.

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

pub use smp::*;
