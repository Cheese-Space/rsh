use crate::builtin::cd;
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
                    Err(_) => {
                        nix::libc::puts(CString::new("executable not found on PATH").unwrap().as_ptr());
                        nix::libc::exit(127);
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
        "cd" => cd(args[1].as_c_str().to_str().unwrap()),
        _ => unreachable!()
    }
}