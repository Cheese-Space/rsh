use crate::status;
use crate::builtin;
use std::ffi::CString;
use std::process::exit;
use std::io;
use nix::errno::Errno;
use nix::sys::signal::kill;
use nix::unistd::Pid;
use nix::unistd::pipe;
use nix::fcntl::{self};
use nix::unistd::write;
use nix::{sys::{wait::{waitpid, WaitStatus}, signal, stat::Mode}, unistd::{read, execvp, fork, ForkResult, dup, dup2_stdout, dup2_stdin}};
const BUILTIN: [&str; 4] = ["exit", "ver", "cd", "mkconf"];
pub fn execute(arguments: Vec<CString>) -> status::ShellResult {
    for i in BUILTIN {
        if i == arguments[0].to_str().unwrap() {
            return exec_intern(i, &arguments);
        }
    }
    for (i, j) in arguments.iter().enumerate() {
        match j.as_bytes() {
            b">" => {
                let filename = match arguments.get(i+1) {
                    Some(s) => s.to_str().unwrap(),
                    None => return Err(status::ShellError::NoArg)
                };
                let arguments = &arguments[0..i];
                return exec_redirect(arguments, filename, true);
            }
            b">>" => {
                let filename = match arguments.get(i+1) {
                    Some(s) => s.to_str().unwrap(),
                    None => return Err(status::ShellError::NoArg)
                };
                let arguments = &arguments[0..i];
                return exec_redirect(arguments, filename, false);
            }
            b"<" => {
                let filename = match arguments.get(i+1) {
                    Some(s) => s.to_str().unwrap(),
                    None => return Err(status::ShellError::NoArg)
                };
                let arguments = &arguments[0..i];
                return exec_file_as_stdin(arguments, filename);
            }
            b"|" => {
                let args1 = &arguments[..i];
                if let None = arguments.get(i+1) {
                    return Err(status::ShellError::NoArg);
                }
                let args2 = &arguments[i+1..];
                return exec_pipe(args1, args2);
            }
            _ => ()
        }
    }
    exec_extern(&arguments)
}
fn exec_extern(arguments: &[CString]) -> status::ShellResult {
    let (e_read, e_write) = pipe().unwrap();
    fcntl::fcntl(&e_write, fcntl::FcntlArg::F_SETFD(fcntl::FdFlag::FD_CLOEXEC)).unwrap();
    unsafe { // note that all libc functions (and fork) are 'unsafe', but won't cause undefined behavior in this code
        match fork() {
            Ok(ForkResult::Parent { child }) => {
                drop(e_write);
                let res = waitpid(child, None).unwrap();
                let mut buff = [0u8; 4];
                let bytes = read(e_read, &mut buff).unwrap();
                if bytes == 4 {
                    let error = i32::from_ne_bytes(buff);
                    return Err(status::ShellError::Exec(Errno::from_raw(error)));
                }
                match res {
                    WaitStatus::Exited(_, code) => {
                        Ok(status::Returns::Code(code))
                    }
                    WaitStatus::Signaled(_, signal, _) => Ok(status::Returns::ShellSignal(signal)),
                    _ => Ok(status::Returns::Code(0))
                }
            }
            Ok(ForkResult::Child) => {
                drop(e_read);
                match execvp(&arguments[0], arguments) {
                    Err(error) => {
                        let error = error as i32;
                        let error = error.to_ne_bytes();
                        write(&e_write, &error).unwrap();
                        drop(e_write);
                        exit(1);
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
                return Err(status::ShellError::NoArg);
            }
            builtin::cd(args[1].to_str().unwrap())
        }
        "mkconf" => {
            crate::config::Conf::make_conf();
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
    if let Err(error) = dup2_stdout(file) {
        return Err(status::ShellError::DupStdout(error, filename.to_string()));
    }
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
    if let Err(error) = dup2_stdin(file) {
        return Err(status::ShellError::DupStdin(error, filename.to_string()));
    }
    let res = exec_extern(arguments);
    dup2_stdin(saved_stdin).expect("couldn't restore stdin!");
    res
}
fn exec_pipe(args1: &[CString], args2: &[CString]) -> status::ShellResult {
    let saved_stdin = dup(io::stdin()).unwrap();
    let saved_stdout = dup(io::stdout()).unwrap();
    let (le_read, le_write) = pipe().unwrap();
    let (re_read, re_write) = pipe().unwrap();
    fcntl::fcntl(&le_write, fcntl::FcntlArg::F_SETFD(fcntl::FdFlag::FD_CLOEXEC)).unwrap();
    fcntl::fcntl(&re_write, fcntl::FcntlArg::F_SETFD(fcntl::FdFlag::FD_CLOEXEC)).unwrap();
    let (read_fd, write_fd) = pipe().unwrap();
    let left_pid: Pid = match unsafe {fork()} {
        Ok(ForkResult::Child) => {
            drop(read_fd);
            drop(le_read);
            dup2_stdout(&write_fd).unwrap();
            drop(write_fd);
            unsafe { signal::signal(signal::Signal::SIGPIPE, signal::SigHandler::SigDfl).unwrap() };
            match execvp(&args1[0], args1) {
                Err(error) => {
                    let error = error as i32;
                    let error = error.to_ne_bytes();
                    write(&le_write, &error).unwrap();
                    exit(1);
                }
            }
        }
        Ok(ForkResult::Parent { child }) => child,
        Err(error) => return Err(status::ShellError::Fork(error))
    };
    let right_pid: Pid = match unsafe {fork()} {
        Ok(ForkResult::Child) => {
            drop(write_fd);
            drop(re_read);
            dup2_stdin(&read_fd).unwrap();
            drop(read_fd);
            unsafe { signal::signal(signal::Signal::SIGPIPE, signal::SigHandler::SigDfl).unwrap() };
            match execvp(&args2[0], args2) {
                Err(error) => {
                    let error = error as i32;
                    let error = error.to_ne_bytes();
                    write(&re_write, &error).unwrap();
                    exit(1);
                }
            }
        }
        Ok(ForkResult::Parent { child }) => child,
        Err(error) => return Err(status::ShellError::Fork(error))
    };
    drop(read_fd);
    drop(write_fd);
    drop(le_write);
    drop(re_write);
    waitpid(left_pid, None).unwrap();
    let mut left_buff = [0u8; 4];
    let bytes = read(le_read, &mut left_buff).unwrap();
    if bytes == 4 {
        kill(right_pid, nix::sys::signal::SIGKILL).unwrap();
        let error = i32::from_ne_bytes(left_buff);
        return Err(status::ShellError::Exec(Errno::from_raw(error)));
    }
    let res = waitpid(right_pid, None).unwrap();
    let mut right_buff = [0u8; 4];
    let bytes = read(re_read, &mut right_buff).unwrap();
    if bytes == 4 {
        let _ = kill(left_pid, nix::sys::signal::SIGKILL);
        let error = i32::from_ne_bytes(right_buff);
        return Err(status::ShellError::Exec(Errno::from_raw(error)));
    }
    dup2_stdin(saved_stdin).expect("couldn't restore stdin!");
    dup2_stdout(saved_stdout).expect("couldn't restore stdout!");
    match res {
        WaitStatus::Exited(_, code) => return Ok(status::Returns::Code(code)),
        WaitStatus::Signaled(_, signal , _) => return Ok(status::Returns::ShellSignal(signal)),
        _ => return  Ok(status::Returns::Code(0))
    }
}