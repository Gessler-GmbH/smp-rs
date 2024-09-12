# SMP
An implementation of the 
[SMP protocol](https://docs.zephyrproject.org/latest/services/device_mgmt/smp_protocol.html) 
in pure Rust.  

This repository contains:
* [./mcumgr-smp](./mcumgr-smp): A SMP library implementation to be used in your own projects
* [./smp-tool](./smp-tool): A command line tool
for some common operations over different transports. 

# Library Usage
The [mcumgr-smp Readme](mcumgr-smp/README.md) contains some usage examples.   
Additionally, you can take a look at the smp-tool code for how to use the library:  
* [Serial transport](./smp-tool/src/transport/serial.rs)
* [Command handling](./smp-tool/src/main.rs)

# License
This project is dual-licensed under 
[MIT](./LICENSE-MIT) and [Apache-2](LICENSE-APACHE)
licenses.  
Copyright (c) 2024 Gessler GmbH.