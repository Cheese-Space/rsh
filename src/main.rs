use std::{io::{self, Write}, process::ExitCode};
use nix::unistd::{User, getuid, gethostname};
use termion::color;
mod exec;
mod parse;
mod status;
mod builtin;
mod config;
fn main() -> ExitCode {
    let mut ucolor = String::new();
    let mut ecolor = String::new();
    let conf = config::Conf::make_conf(true);
    match conf.usercolor {
        0 => ucolor.push_str(&color::Fg(color::LightGreen).to_string()),
        _ => ()
    }
    match conf.errorcolor {
        0 => ecolor.push_str(&color::Fg(color::LightRed).to_string()),
        _ => ()
    }
    let user_info = User::from_uid(getuid()).unwrap().unwrap();
    let username = user_info.name;
    let hostname = gethostname().unwrap();
    let hostname = hostname.to_str().unwrap();
    let mut return_code = 0;
    loop {
        if return_code == 0 {
            print!("{}{}{}@{} {} ", ucolor, username, color::Fg(color::Reset), hostname, conf.separator);
        }
        else {
            print!("{}{}{}@{} [{}{}{}] {} ", ucolor, username, color::Fg(color::Reset), hostname, ecolor, return_code, color::Fg(color::Reset), conf.separator);
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
                println!("{}", val);
                return_code = 1;
            }
        }
    }
}