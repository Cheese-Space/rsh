use crate::status;
use crate::builtin;
use std::ffi::CString;
use std::process::exit;
use std::io;
use nix::errno::Errno;
use nix::{sys::{wait::{waitpid, WaitStatus}, stat::Mode}, unistd::{execvp, fork, ForkResult, dup, dup2_stdout, dup2_stdin}, fcntl};
const BUILTIN: [&str; 4] = ["exit", "ver", "cd", "mkconf"];
pub fn execute(arguments: Vec<CString>) -> status::ShellResult {
    for i in BUILTIN {
        if i == arguments[0].to_str().unwrap() {
            return exec_intern(i, &arguments);
        }
    }
    for (i, j) in arguments.iter().enumerate() {
        if j.as_bytes() == b">" {
            let filename = arguments[i+1].to_str().unwrap();
            let arguments = &arguments[0..i];
            return exec_redirect(arguments, filename, true);
        }
        else if j.as_bytes() == b">>" {
            let filename = arguments[i+1].to_str().unwrap();
            let arguments = &arguments[0..i];
            return exec_redirect(arguments, filename, false);
        }
        else if j.as_bytes() == b"<" {
             let filename = arguments[i+1].to_str().unwrap();
            let arguments = &arguments[0..i];
            return exec_file_as_stdin(arguments, filename);
        }
    }
    exec_extern(&arguments)
}
fn exec_extern(arguments: &[CString]) -> status::ShellResult {
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
                        eprintln!("error: {}", error.desc());
                        if error == Errno::ENOENT {
                            exit(127);
                        }
                        else {
                            exit(1);
                        }
                    }
                }
            }
            Err(error) => Err(status::ShellError::Fork(error))
        }
    }
}
fn exec_intern(func: &str, args:&[CString]) -> status::ShellResult {
    match func {
        "exit" => Ok(builtin::exit()),
        "ver" => Ok(builtin::version()),
        "cd" => {
            if args.len() < 2 {
                eprintln!("error: no input provided!");
                return Ok(status::Returns::Code(1));
            }
            builtin::cd(args[1].to_str().unwrap())
        }
        "mkconf" => {
            crate::config::Conf::make_conf(false);
            Ok(status::Returns::Code(0))
        }
        _ => unreachable!()
    }
}
fn exec_redirect(arguments: &[CString], filename: &str, overwrite: bool) -> status::ShellResult {
    let stdout = io::stdout();
    let saved_stdout = dup(&stdout).unwrap();
    let mut flags = fcntl::OFlag::O_WRONLY | fcntl::OFlag::O_CLOEXEC | fcntl::OFlag::O_APPEND;
    if overwrite {
        flags = fcntl::OFlag::O_WRONLY | fcntl::OFlag::O_CREAT | fcntl::OFlag::O_TRUNC | fcntl::OFlag::O_CLOEXEC;
    }
    let file = match fcntl::open(filename, flags, Mode::S_IWUSR | Mode::S_IRUSR) {
        Ok(f) => f,
        Err(error) => return Err(status::ShellError::IO(error))
    };
    dup2_stdout(file).unwrap();
    let res = exec_extern(arguments);
    dup2_stdout(saved_stdout).expect("couldn't restore stdout!");
    res
}
fn exec_file_as_stdin(arguments: &[CString], filename: &str) -> status::ShellResult {
    let stdin = io::stdin();
    let saved_stdin = dup(&stdin).unwrap();
    let file = match fcntl::open(filename, fcntl::OFlag::O_RDONLY | fcntl::OFlag::O_CLOEXEC, Mode::empty()) {
        Ok(f) => f,
        Err(error) => return Err(status::ShellError::IO(error))
    };
    dup2_stdin(file).unwrap();
    let res = exec_extern(arguments);
    dup2_stdin(saved_stdin).expect("couldn't restore stdin!");
    res
}