use std::{io::{self, Write}, process::ExitCode};
use nix::unistd::{User, getuid, gethostname};
use termion::color;
mod exec;
mod parse;
mod status;
mod builtin;
mod config;
fn main() -> ExitCode {
    let conf = config::Conf::read_conf();
    let user_info = User::from_uid(getuid()).unwrap().unwrap();
    let username = user_info.name;
    let hostname = gethostname().unwrap();
    let hostname = match hostname.into_string() {
        Ok(s) => s,
        Err(_) => {
            eprintln!("error: hostname contains invalid UTF-8\nsetting hostname to: 'unknown'");
            String::from("unknown")
        }
    };
    let mut return_code = 0;
    loop {
        if return_code == 0 {
            print!("{}{}{}@{} {} ", conf.usercolor, username, color::Fg(color::Reset), hostname, conf.separator);
        }
        else {
            print!("{}{}{}@{} [{}{}{}] {} ", conf.usercolor, username, color::Fg(color::Reset), hostname, conf.errorcolor, return_code, color::Fg(color::Reset), conf.separator);
        }
        io::stdout().flush().unwrap();
        let input = parse::parse_input();
        if input.trim().is_empty() {
            continue;
        } 
        let args = match parse::split_input(&input) {
            Ok(s) => s,
            Err(_) => {
                eprintln!("{}", status::ShellError::CStringNullByte);
                continue;
            }
        };
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
                eprintln!("{}", val);
                return_code = 1;
            }
        }
    }
}