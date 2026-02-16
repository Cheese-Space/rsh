use std::process::ExitCode;
use nix::{sys::signal::{Signal, signal}, unistd::{User, gethostname, getuid}};
use rustyline::{DefaultEditor, error::ReadlineError};
use termion::color;
mod exec;
mod parse;
mod status;
mod builtin;
mod config;
fn main() -> ExitCode {
    unsafe {
        signal(Signal::SIGINT, nix::sys::signal::SigHandler::SigIgn).unwrap();
        signal(Signal::SIGPIPE, nix::sys::signal::SigHandler::SigIgn).unwrap();
        signal(Signal::SIGTTOU, nix::sys::signal::SigHandler::SigIgn).unwrap();
        signal(Signal::SIGTTIN, nix::sys::signal::SigHandler::SigIgn).unwrap();
    }
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
    let mut signal: Option<Signal> = None;
    let mut rl = DefaultEditor::new().unwrap();
    loop {
        let readline: Result<String, ReadlineError>;
        if let Some(sig) = signal {
            if sig == Signal::SIGINT {
                println!();
            }
            readline = rl.readline(&format!("{}{}{}@{} [{}{}{}] {} ", conf.usercolor, username, color::Fg(color::Reset), hostname, conf.errorcolor, sig, color::Fg(color::Reset), conf.separator));
        }
        else if  return_code == 0 {
            readline = rl.readline(&format!("{}{}{}@{} {} ", conf.usercolor, username, color::Fg(color::Reset), hostname, conf.separator));
        }
        else {
            readline = rl.readline(&format!("{}{}{}@{} [{}{}{}] {} ", conf.usercolor, username, color::Fg(color::Reset), hostname, conf.errorcolor, return_code, color::Fg(color::Reset), conf.separator));
        }
        let input = match readline {
            Ok(s) => {
                rl.add_history_entry(s.as_str()).unwrap();
                s
            }
            Err(ReadlineError::Interrupted) => continue,
            Err(ReadlineError::Eof) => {
                eprintln!("reached EOF"); 
                break;
            }
            Err(err) => {
                eprintln!("{:?}", err); 
                continue;
            }
        };
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
                        signal = None;
                    }
                    status::Returns::ShellSignal(sig) => signal = Some(sig),
                    status::Returns::ExitSig => return ExitCode::SUCCESS
                }
            }
            Err(val) => {
                eprintln!("{}", val);
                if let status::ShellError::Exec(error) = val {
                    if error == nix::errno::Errno::ENOENT {
                        return_code = 127;
                    }
                    else {
                        return_code = 126;
                    }
                }
                else {
                    return_code = 1;
                }
                signal = None;
            }
        }
    }
    ExitCode::FAILURE
}
