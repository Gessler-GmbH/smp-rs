# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased
No changes since 0.8.0

## [0.8.0] - 2025-01-08

### Added
- Add [`upgrade`](https://docs.zephyrproject.org/latest/services/device_mgmt/smp_groups/smp_group_1.html#image-upload-request) flag to image upload request
- Add parsing of [`rsn`](https://docs.zephyrproject.org/latest/services/device_mgmt/smp_groups/smp_group_1.html#image-upload-response) error string fields where applicable
- Add parsing of [`match`](https://docs.zephyrproject.org/latest/services/device_mgmt/smp_groups/smp_group_1.html#image-upload-response) field of image upload response, for on-device checksum verification
- Add optional check for correct sequence number in `receive_cbor`/`transceive_cbor` functions

### Changed
- Updated documentation to include info on the now included transports
- Reduce default chunk size to `256` to match Zephyr's default
- Take request data in `send_cbor` and `transceive_cbor` as reference, which allows users to implement a retry mechanism
- Update dependencies in Cargo.lock

### Fixed
- Fix broken messages after merging of multi-chunk responses

### Removed
- Unused `regex` dependency

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
