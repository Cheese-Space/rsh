use std::{io::{self, Write}, process::ExitCode};
use nix::unistd::{User, getuid, gethostname};
mod exec;
mod parse;
mod status;
mod builtin;
fn main() -> ExitCode {
    let user_info = User::from_uid(getuid()).unwrap().unwrap();
    let username = user_info.name;
    let hostname = gethostname().unwrap();
    let hostname = hostname.to_str().unwrap();
    let mut return_code = status::ReturnCode(0);
    loop {
        print!("{}@{}{}", username, hostname, return_code);
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
                println!("{}", val);
                return_code = status::ReturnCode(1);
            }
        }
    }
}