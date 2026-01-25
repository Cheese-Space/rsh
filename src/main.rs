use std::{io::{self, Write}, process::ExitCode};
mod exec;
mod parse;
mod status;
mod builtin;
fn main() -> ExitCode {
    let mut return_code = status::ReturnCode(0);
    loop {
        print!("{}", return_code);
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
                status::ReturnCode(1);
            }
        }
    }
}