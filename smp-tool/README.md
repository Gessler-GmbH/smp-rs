# SMP Tool
Command-line tool to send and receive SMP messages.


## Installation
Installation and building is easily done using cargo:
```shell
cargo install --path smp-tool
```

## Usage
Serial Backend:
```shell
smp-tool -t serial -s /dev/ttyACM0 os echo "hello world SMP"
```

UDP Backend:
```shell
smp-tool -t udp -i "2001:db8::1" os echo "hello world SMP"
```

Updating Firmware:
```shell
smp-tool -t serial -s /dev/ttyACM0 app flash -c 512 -u ./zephyr.signed.bin
```

Start an interactive shell over SMP:
```shell
smp-tool -t serial -s /dev/ttyACM0 shell interactive
```




Copyright (c) 2023 Gessler GmbH.