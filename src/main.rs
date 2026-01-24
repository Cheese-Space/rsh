use std::{io::{self, Write}, process::ExitCode};
use nix::errno::Errno;
use termion::color;

use crate::status::ShellError;
mod exec;
mod parse;
mod status;
mod builtin;
fn main() -> ExitCode {
    let mut return_code: Option<i32> = None;
    loop {
        match return_code {
            Some(val) => print!("[{}{}{}]> ", color::Fg(color::LightRed), val, color::Fg(color::Reset)),
            None => print!("> ")
        }
        io::stdout().flush().unwrap();
        let input = parse::parse_input();
        if input.trim().is_empty() {
            continue;
        }
        let args = parse::split_input(&input);
        match exec::execute(args) {
            Ok(code) => {
                match code {
                    status::Returns::Code(co) => {
                        return_code = co;
                    }
                    status::Returns::ShellSignal(sig) => {
                        todo!();
                    }
                    status::Returns::ExitSig => return ExitCode::SUCCESS
                }
            }
            Err(val) => {
                match val {
                    ShellError::IO(error) => {
                        match error {
                            Errno::ENOENT => println!("specified dir doesn't exit"),
                            Errno::ENOTDIR => println!("file exists, but isn't a dir"),
                            _ => ()
                        }
                    }
                    ShellError::Fork => todo!()
                }
                return_code = Some(1);
            }
        }
    }
}
