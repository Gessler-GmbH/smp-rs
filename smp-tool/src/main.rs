// Author: Sascha Zenglein <zenglein@gessler.de>
// Copyright (c) 2023 Gessler GmbH.

use std::cmp::min;
use std::error::Error;
use std::path::PathBuf;
use std::time::Duration;

use clap::{Parser, Subcommand, ValueEnum};
use mcumgr_smp::transport::udp::AsyncUDPTransport;
use mcumgr_smp::transport::AsyncCBORSMPTransport;
use mcumgr_smp::{
    application_management, application_management::GetImageStateResult,
    application_management::WriteImageChunkResult, os_management, os_management::EchoResult,
    shell_management, shell_management::ShellResult, SMPFrame,
};
use sha2::Digest;
use tracing::debug;
use tracing_subscriber::prelude::*;

/// interactive shell support
pub mod shell;

#[derive(ValueEnum, Copy, Clone, Debug)]
pub enum Transport {
    Serial,
    UDP,
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cli: Cli = Cli::parse();

    let transport: Box<dyn mcumgr_smp::transport::AsyncSMPTransport + Unpin> = match cli.transport {
        Transport::Serial => {
            let serial = mcumgr_smp::transport::serial::AsyncSerialTransport::open(
                cli.serial_device.expect("serial device is expected"),
                cli.serial_baud,
            )?;
            Box::new(serial)
        }
        Transport::UDP => {
            let host = cli.dest_host.expect("dest_host required");
            let port = cli.udp_port;

            debug!("connecting to {} at port {}", host, port);

            let udp = AsyncUDPTransport::new((host, port)).await?;
            Box::new(udp)
        }
    };
    let mut transport = AsyncCBORSMPTransport::new(transport);

    //transport.set_recv_timeout(Some(Duration::from_millis(cli.timeout_ms)))?;

    match cli.command {
        Commands::Os(OsCmd::Echo { msg }) => {
            let ret: SMPFrame<EchoResult> =
                transport.transceive(os_management::echo(42, msg)).await?;
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
            let ret: SMPFrame<ShellResult> = transport
                .transceive(shell_management::shell_command(42, cmd))
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

                let frame = updater.write_chunk(chunk);

                let resp_frame: SMPFrame<WriteImageChunkResult> =
                    transport.transceive(frame).await?;

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
            let ret: SMPFrame<GetImageStateResult> = transport
                .transceive(application_management::get_state(42))
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
