use crate::status;
use crate::builtin;
use std::ffi::CString;
use nix::{sys::wait::{waitpid, WaitStatus}, unistd::{execvp, fork, ForkResult}};
const BUILTIN: [&str; 3] = ["exit", "ver", "cd"];
pub fn execute(arguments: Vec<CString>) -> status::ShellResult {
    for i in BUILTIN {
        if i == String::from(arguments[0].to_str().unwrap()) {
            return exec_intern(i, arguments);
        }
    }
    exec_extern(arguments)
}
fn exec_extern(arguments: Vec<CString>) -> status::ShellResult {
    unsafe { // note that all libc functions (and fork) are 'unsafe', but won't cause undefined behavior in this code
        match fork() {
            Ok(ForkResult::Parent { child }) => {
                match waitpid(child, None).unwrap() {
                    WaitStatus::Exited(_, code) => {
                        if code == 0 {
                            Ok(status::Returns::Code(status::ReturnCode(0)))
                        }
                        else {
                            Ok(status::Returns::Code(status::ReturnCode(code)))
                        }
                    },
                    WaitStatus::Signaled(_, signal, _) => Ok(status::Returns::ShellSignal(signal)),
                    _ => Ok(status::Returns::Code(status::ReturnCode(0)))
                }

            }
            Ok(ForkResult::Child) => {
                match execvp(&arguments[0], &arguments) {
                    Ok(_) => unreachable!(),
                    Err(error) => {
                        if let nix::errno::Errno::ENOENT = error {
                            println!("file not found on PATH");
                            nix::libc::exit(127);
                        }
                        else {
                            println!("{}", error);
                            nix::libc::exit(1);
                        }
                    }
                }
            }
            Err(_) => Err(status::ShellError::Fork)
        }
    }
}
fn exec_intern(func: &str, args: Vec<CString>) -> status::ShellResult {
    match func {
        "exit" => Ok(builtin::exit()),
        "ver" => Ok(builtin::version()),
        "cd" => builtin::cd(args[1].as_c_str().to_str().unwrap()),
        _ => unreachable!()
    }
}