use std::{io::{self, Write}, thread::sleep, time::Duration, process::ExitCode};
use nix::errno::Errno;
mod exec;
mod parse;
mod status;
mod builtin;
fn main() -> ExitCode {
    let mut return_code: Option<i32> = None;
    loop {
        match return_code {
            Some(val) => print!("[{}]> ", val),
            None => print!("> ")
        }
        io::stdout().flush().unwrap();
        let input = parse::parse_input();
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
                    status::ShellError::Execv(err) => {
                        match err {
                            Errno::ENOENT => {
                                println!("executable not found on PATH");
                                return_code = Some(127);
                            }
                            _ => ()
                        }
                    }
                    _ => ()
                }
            }
        }
        sleep(Duration::from_millis(1));
    }
}
