// Author: Sascha Zenglein <zenglein@gessler.de>
// Copyright (c) 2023 Gessler GmbH.

use std::cmp::min;
use std::error::Error;
use std::path::PathBuf;
use std::time::Duration;

use clap::{Parser, Subcommand, ValueEnum};
use mcumgr_smp::{
    application_management::{self, GetImageStateResult, WriteImageChunkResult},
    os_management::{self, EchoResult},
    shell_management::{self, ShellResult},
    smp::SmpFrame,
    transport::{
        ble::BleTransport,
        serial::SerialTransport,
        smp::{CborSmpTransport, CborSmpTransportAsync},
        udp::UdpTransportAsync,
    },
};
use sha2::Digest;
use tracing::debug;
use tracing_subscriber::prelude::*;

/// interactive shell support
pub mod shell;

#[derive(ValueEnum, Copy, Clone, Debug)]
pub enum Transport {
    Serial,
    Udp,
    Ble,
}

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "Command-line tool to send and receive SMP messages.",
    before_help = "Copyright (c) 2023 Gessler GmbH.",
    help_template = "{about-with-newline}\nAuthor: {author-with-newline}{before-help}{usage-heading} {usage}\n\n{all-args}"
)]
struct Cli {
    #[arg(short, long, value_enum)]
    transport: Transport,

    #[arg(short, long, required_if_eq("transport", "serial"))]
    serial_device: Option<String>,

    #[arg(short = 'b', long, default_value_t = 115200)]
    serial_baud: u32,

    #[arg(short = 'd', long, required_if_eq("transport", "udp"))]
    dest_host: Option<String>,

    #[arg(short = 'p', long, default_value_t = 1337)]
    udp_port: u16,

    #[arg(long, default_value = "5000")]
    timeout_ms: u64,

    #[arg(short, long, required_if_eq("transport", "ble"))]
    name: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Send a command in the os group
    #[command(subcommand)]
    Os(OsCmd),
    /// Send a command in the shell group
    #[command(subcommand)]
    Shell(ShellCmd),
    /// Send a command in the application group
    #[command(subcommand)]
    App(ApplicationCmd),
}

#[derive(Subcommand, Debug)]
enum OsCmd {
    /// Send an SMP Echo request
    Echo { msg: String },
}
#[derive(Subcommand, Debug)]
enum ShellCmd {
    /// Send a shell command via SMP and read the response
    Exec { cmd: Vec<String> },
    /// Start a remote interactive shell using SMP as the backend
    Interactive,
}
#[derive(Subcommand, Debug)]
enum ApplicationCmd {
    /// Request firmware info
    Info,
    // /// Erase a partition
    // Erase {
    //     #[arg(short, long)]
    //     slot: u8,
    // },
    /// Flash a firmware to an image slot
    Flash {
        #[arg(short, long)]
        slot: Option<u8>,
        #[arg(short, long)]
        update_file: PathBuf,
        #[arg(short, long, default_value_t = 512)]
        chunk_size: usize,
    },
}

pub enum UsedTransport {
    SyncTransport(CborSmpTransport),
    AsyncTransport(CborSmpTransportAsync),
}

impl UsedTransport {
    pub async fn transceive_cbor<Req: serde::Serialize, Resp: serde::de::DeserializeOwned>(
        &mut self,
        frame: SmpFrame<Req>,
    ) -> Result<SmpFrame<Resp>, mcumgr_smp::transport::error::Error> {
        match self {
            UsedTransport::SyncTransport(ref mut t) => t.transceive_cbor(frame),
            UsedTransport::AsyncTransport(ref mut t) => t.transceive_cbor(frame).await,
        }
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cli: Cli = Cli::parse();

    let mut transport = match cli.transport {
        Transport::Serial => {
            let mut t = SerialTransport::new(
                cli.serial_device.expect("serial device required"),
                cli.serial_baud,
            )?;
            t.recv_timeout(Some(Duration::from_millis(cli.timeout_ms)))?;
            UsedTransport::SyncTransport(CborSmpTransport {
                transport: Box::new(t),
            })
        }
        Transport::Udp => {
            let host = cli.dest_host.expect("dest_host required");
            let port = cli.udp_port;

            debug!("connecting to {} at port {}", host, port);

            UsedTransport::AsyncTransport(CborSmpTransportAsync {
                transport: Box::new(UdpTransportAsync::new((host, port)).await?),
            })
        }
        Transport::Ble => {
            let adapters = BleTransport::adapters().await?;
            debug!("found {} adapter(s): {:?}:", adapters.len(), adapters);
            let adapter = adapters.first().ok_or("BLE adapters not found")?;
            debug!("selecting first adapter: {:?}:", adapter);
            UsedTransport::AsyncTransport(CborSmpTransportAsync {
                transport: Box::new(
                    BleTransport::new(
                        cli.name.unwrap(),
                        adapter,
                        Duration::from_millis(cli.timeout_ms),
                    )
                    .await?,
                ),
            })
        }
    };

    match cli.command {
        Commands::Os(OsCmd::Echo { msg }) => {
            let ret: SmpFrame<EchoResult> = transport
                .transceive_cbor(os_management::echo(42, msg))
                .await?;
            debug!("{:?}", ret);

            match ret.data {
                EchoResult::Ok { r } => {
                    println!("{}", r);
                }
                EchoResult::Err { rc } => {
                    eprintln!("rc: {}", rc);
                }
            }
        }
        Commands::Shell(ShellCmd::Exec { cmd }) => {
            let ret: SmpFrame<ShellResult> = transport
                .transceive_cbor(shell_management::shell_command(42, cmd))
                .await?;
            debug!("{:?}", ret);

            match ret.data {
                ShellResult::Ok { o, ret } => {
                    println!("ret: {}, o: {}", ret, o);
                }
                ShellResult::Err { rc } => {
                    eprintln!("rc: {}", rc);
                }
            }
        }
        Commands::Shell(ShellCmd::Interactive) => {
            shell::shell(&mut transport).await?;
        }
        Commands::App(ApplicationCmd::Flash {
            slot,
            update_file,
            chunk_size,
        }) => {
            let firmware = std::fs::read(&update_file)?;

            let mut hasher = sha2::Sha256::new();
            hasher.update(&firmware);
            let hash = hasher.finalize();

            debug!("Image sha256: {:02X?}", hash);

            let mut updater = mcumgr_smp::application_management::ImageWriter::new(
                slot,
                firmware.len(),
                Some(&hash),
            );

            let mut offset = 0;
            while offset < firmware.len() {
                println!("writing {}/{}", offset, firmware.len());
                let chunk = &firmware[offset..min(firmware.len(), offset + chunk_size)];

                let resp_frame: SmpFrame<WriteImageChunkResult> = transport
                    .transceive_cbor(updater.write_chunk(chunk))
                    .await?;

                match resp_frame.data {
                    WriteImageChunkResult::Ok(payload) => {
                        offset = payload.off as usize;
                        updater.offset = offset;
                    }
                    WriteImageChunkResult::Err(err) => {
                        Err(format!("Err from MCU: {:?}", err).to_string())?
                    }
                }
            }

            println!("sent all bytes: {}", offset);
        }
        Commands::App(ApplicationCmd::Info) => {
            let ret: SmpFrame<GetImageStateResult> = transport
                .transceive_cbor(application_management::get_state(42))
                .await?;
            debug!("{:?}", ret);

            match ret.data {
                GetImageStateResult::Ok(payload) => {
                    println!("{:?}", payload)
                }
                GetImageStateResult::Err(err) => {
                    eprintln!("rc: {}", err.rc);
                }
            }
        }
    }
    Ok(())
}
