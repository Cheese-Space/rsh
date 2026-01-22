use crate::status;
use std::ffi::CString;
use nix::{sys::wait::{waitpid, WaitStatus}, unistd::{execvp, fork, ForkResult}};
pub fn execute(arguments: Vec<CString>) -> status::ShellResult {
    exec_extern(arguments)
}
fn exec_extern(arguments: Vec<CString>) -> status::ShellResult {
    unsafe { // note that fork() is the only unsafe function
        match fork() {
            Ok(ForkResult::Parent { child }) => {
                match waitpid(child, None).unwrap() {
                    WaitStatus::Exited(_, code) => Ok(status::Returns::Code(code)),
                    WaitStatus::Signaled(_, signal, _) => Ok(status::Returns::ShellSignal(signal)),
                    _ => Ok(status::Returns::Code(1))
                }

            }
            Ok(ForkResult::Child) => {
                match execvp(&arguments[0], &arguments) {
                    Err(_) => Err(status::ShellError::Execv)
                }
            }
            Err(_) => Err(status::ShellError::Fork)
        }
    }
}