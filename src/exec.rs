use crate::status;
use crate::builtin;
use std::ffi::CString;
use std::process::exit;
use nix::errno::Errno;
use nix::{sys::wait::{waitpid, WaitStatus}, unistd::{execvp, fork, ForkResult}};
const BUILTIN: [&str; 3] = ["exit", "ver", "cd"];
pub fn execute(arguments: Vec<CString>) -> status::ShellResult {
    for i in BUILTIN {
        if i == arguments[0].to_str().unwrap() {
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
                        Ok(status::Returns::Code(code))
                    },
                    WaitStatus::Signaled(_, signal, _) => Ok(status::Returns::ShellSignal(signal)),
                    _ => Ok(status::Returns::Code(0))
                }
            }
            Ok(ForkResult::Child) => {
                match execvp(&arguments[0], &arguments) {
                    Err(error) => {
                        println!("error: {}", error.desc());
                        if error == Errno::ENOENT {
                            exit(127);
                        }
                        else {
                            exit(1);
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
        "cd" => {
            if args.len() < 2 {
                println!("error: no input provided!");
                return Ok(status::Returns::Code(1));
            }
            builtin::cd(args[1].to_str().unwrap())
        }
        _ => unreachable!()
    }
}