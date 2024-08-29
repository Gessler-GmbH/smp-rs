# mcumgr-smp
An implementation of the 
[SMP protocol](https://docs.zephyrproject.org/latest/services/device_mgmt/smp_protocol.html).  

This library defines the SMP message format and methods to encode and decode bytes.

Additionally, common request and response objects with a CBOR payload for groups
Application Management, OS Management, and Shell Management are provided.  
Feel free to contribute additional message definitions.

**Custom messages are fully supported by creating SMPFrames manually.
You can even use a payload encoding other than CBOR.**

By default it include all available transport. If you don't need them all, disable default features
and enable the needed one.

## Example
Echo
```rust
// build an echo SMP message
let smp_frame: mcumgr_smp::SmpFrame = mcumgr_smp::os_management::echo(42, "Hello World");
// get frame as bytes
let data: Vec<u8> = smp_frame.encode_with_cbor();
// send frame and get response
// ...
let response_data = [3, 0, 0, 16, 0, 0, 66, 0, 191, 97, 114, 107, 72, 101, 108, 108, 111, 32, 87, 111, 114, 108, 100, 255];
let response: SmpFrame<EchoResult> = mcumgr_smp::SmpFrame::decode_with_cbor(&response_data).expect("decoding error");

println!("response payload: {:?}", response.data);
```

You can also create a frame manually:
```rust
let data = todo!(); // cbor payload of command, in this case a firmware chunk
let smp_frame = SmpFrame::new(
    mcumgr_smp::OpCode::WriteRequest,
    69,
    mcumgr_smp::Group::ApplicationManagement,
    mcumgr_smp::ApplicationManagementCommand::Upload as u8,
    data,
);
```




Copyright (c) 2023 Gessler GmbH.