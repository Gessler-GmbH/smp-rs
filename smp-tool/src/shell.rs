// Author: Sascha Zenglein <zenglein@gessler.de>
// Copyright (c) 2023 Gessler GmbH.

use std::error::Error;

use reedline::{
    default_emacs_keybindings, DefaultPrompt, DefaultPromptSegment, Emacs, Reedline, Signal,
};
use tracing::debug;

use mcumgr_smp::{shell_management, shell_management::ShellResult, SMPFrame};

use crate::transport::CborSMPTransport;

pub fn shell(transport: &mut CborSMPTransport) -> Result<(), Box<dyn Error>> {
    let keybindings = default_emacs_keybindings();
    let edit_mode = Box::new(Emacs::new(keybindings));

    let prompt = DefaultPrompt::new(
        DefaultPromptSegment::Basic("SMP Shell: ".to_string()),
        DefaultPromptSegment::Empty,
    );

    let mut line_editor = Reedline::create().with_edit_mode(edit_mode);

    loop {
        let sig = line_editor.read_line(&prompt)?;

        match sig {
            Signal::Success(buffer) => 'succ: {
                let argv: Vec<_> = buffer.split_whitespace().map(|s| s.to_owned()).collect();

                let ret: Result<SMPFrame<ShellResult>, _> =
                    transport.transceive_cbor(shell_management::shell_command(42, argv));
                debug!("{:?}", ret);

                let data = match ret {
                    Ok(smp_frame) => smp_frame.data,
                    Err(err) => {
                        println!("transport error: {}", err);
                        break 'succ;
                    }
                };

                match data {
                    ShellResult::Ok { o, ret: _ } => {
                        println!("{}", o);
                    }
                    ShellResult::Err { rc } => {
                        eprintln!("SMP Error: rc: {}", rc);
                    }
                }
            }
            Signal::CtrlD | Signal::CtrlC => {
                println!("\nAborted!");
                break Ok(());
            }
        }
    }
}
