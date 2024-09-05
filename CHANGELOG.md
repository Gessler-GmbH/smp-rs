# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

## [0.7.0] - 2024-09-05

This update has been driven by @aectaan with the primary
goal to add Bluetooth support!

### Added

- The sync SmpTransport trait and the async version SmpTransportAsync
  as a very basic send and receive abstraction.
- CborSmpTransport that handles CBOR decoding for you and wraps a boxed transport implementation
- async UDP, BLE transports
- [smp-tool] parameter to select a bluetooth device as target
- github workflows for linting, formatting and compilation

### Changed

- moved sync UDP and Serial transports into [mcumgr-smp]
- change SMP struct and enum prefix to Smp to be more consistent
  with Rust naming conventions.
- Upgrade dependencies
- [smp-tool] skip to version 0.7 to align with [mcumgr-smp]

## [0.6.0] - 2023-12-07

### Added

- published library and CLI
- SMP Message definitions and parsing
- Firmware update handler
- [smp-tool] Serial and UDP transport
