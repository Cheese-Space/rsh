use crate::status;
use crate::builtin;
use std::ffi::CString;
use nix::{sys::wait::{waitpid, WaitStatus}, unistd::{execvp, fork, ForkResult}, errno};
const BUILTIN: [&str; 2] = ["exit", "ver"];
pub fn execute(arguments: Vec<CString>) -> status::ShellResult {
    for i in BUILTIN {
        if i == String::from(arguments[0].to_str().unwrap()) {
            return exec_intern(i);
        }
    }
    exec_extern(arguments)
}
fn exec_extern(arguments: Vec<CString>) -> status::ShellResult {
    unsafe { // note that fork() is the only unsafe function; also note that fork() is not unsafe in this context as we only call execvp in the child
        match fork() {
            Ok(ForkResult::Parent { child }) => {
                match waitpid(child, None).unwrap() {
                    WaitStatus::Exited(_, code) => {
                        if code == 0 {
                            Ok(status::Returns::Code(None))
                        }
                        else {
                            Ok(status::Returns::Code(Some(code)))
                        }
                    },
                    WaitStatus::Signaled(_, signal, _) => Ok(status::Returns::ShellSignal(signal)),
                    _ => Ok(status::Returns::Code(None))
                }

            }
            Ok(ForkResult::Child) => {
                match execvp(&arguments[0], &arguments) {
                    Err(_) => Err(status::ShellError::Execv(errno::Errno::last()))
                }
            }
            Err(_) => Err(status::ShellError::Fork)
        }
    }
}
fn exec_intern(func: &str) -> status::ShellResult {
    match func {
        "exit" => Ok(builtin::exit()),
        "ver" => Ok(builtin::version()),
        _ => unreachable!()
    }
}