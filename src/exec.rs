use crate::status;
use crate::builtin;
use std::ffi::CString;
use std::process::exit;
use std::io;
use nix::errno::Errno;
use nix::fcntl::OFlag;
use nix::fcntl::open;
use nix::sys::signal::Signal::SIGTERM;
use nix::sys::signal::killpg;
use nix::sys::signal::signal;
use nix::unistd::Pid;
use nix::unistd::getpid;
use nix::unistd::pipe;
use nix::fcntl::{self};
use nix::unistd::setpgid;
use nix::unistd::tcsetpgrp;
use nix::unistd::write;
use nix::sys::signal::Signal;
use nix::{sys::{wait::{waitpid, WaitStatus}, stat::Mode}, unistd::{read, execvp, fork, ForkResult, dup, dup2_stdout, dup2_stdin}};
use rustyline::DefaultEditor;
const BUILTIN: [&str; 5] = ["exit", "ver", "cd", "mkconf", "history"];
macro_rules! set_sig_to_def {
    () => {
        signal(Signal::SIGINT, nix::sys::signal::SigHandler::SigDfl).unwrap();
        signal(Signal::SIGPIPE, nix::sys::signal::SigHandler::SigDfl).unwrap();
        signal(Signal::SIGTTOU, nix::sys::signal::SigHandler::SigDfl).unwrap();
        signal(Signal::SIGTTIN, nix::sys::signal::SigHandler::SigDfl).unwrap();
    };
}
pub fn execute(arguments: Vec<CString>, line_editor: &DefaultEditor) -> status::ShellResult {
    for i in BUILTIN {
        if i == arguments[0].to_str().unwrap() {
            return exec_intern(i, &arguments, line_editor);
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
            b"&&" => {
                let args1 = &arguments[..i];
                if let None = arguments.get(i+1) {
                    return Err(status::ShellError::NoArg);
                }
                let args2 = &arguments[i+1..];
                return exec_and(args1, args2);
            }
            _ => ()
        }
    }
    exec_extern(&arguments)
}
fn exec_extern(arguments: &[CString]) -> status::ShellResult {
    let (e_read, e_write) = pipe().map_err(|error| status::ShellError::Pipe(error))?;
    fcntl::fcntl(&e_write, fcntl::FcntlArg::F_SETFD(fcntl::FdFlag::FD_CLOEXEC)).unwrap();
    unsafe { // note that all libc functions (and fork) are 'unsafe', but won't cause undefined behavior in this code
        match fork() {
            Ok(ForkResult::Parent { child }) => {
                drop(e_write);
                let res: WaitStatus = loop {
                    let res = waitpid(child, None);
                    if let Err(Errno::EINTR) = res {
                        continue;
                    }
                    else if let Ok(val) = res {
                        break val;
                    }
                };
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
                set_sig_to_def!();
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
fn exec_intern(func: &str, args:&[CString], line_editor: &DefaultEditor) -> status::ShellResult {
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
        "history" => {
            Ok(builtin::history(line_editor))
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
    let file = fcntl::open(filename, flags, Mode::S_IWUSR | Mode::S_IRUSR).map_err(|error| status::ShellError::IO(error))?;
    dup2_stdout(file).map_err(|error| status::ShellError::DupStdout(error, filename.to_string()))?;
    let res = exec_extern(arguments);
    dup2_stdout(saved_stdout).expect("couldn't restore stdout!");
    res
}
fn exec_file_as_stdin(arguments: &[CString], filename: &str) -> status::ShellResult {
    let stdin = io::stdin();
    let saved_stdin = dup(&stdin).unwrap();
    let file = fcntl::open(filename, fcntl::OFlag::O_RDONLY | fcntl::OFlag::O_CLOEXEC, Mode::empty()).map_err(|error| status::ShellError::IO(error))?;
    dup2_stdin(file).map_err(|error| status::ShellError::DupStdin(error, filename.to_string()))?;
    let res = exec_extern(arguments);
    dup2_stdin(saved_stdin).expect("couldn't restore stdin!");
    res
}
fn exec_pipe(args1: &[CString], args2: &[CString]) -> status::ShellResult {
    let saved_stdin = dup(io::stdin()).unwrap();
    let saved_stdout = dup(io::stdout()).unwrap();
    let (le_read, le_write) = pipe().map_err(|error| status::ShellError::Pipe(error))?;
    let (re_read, re_write) =  pipe().map_err(|error| status::ShellError::Pipe(error))?;
    fcntl::fcntl(&le_write, fcntl::FcntlArg::F_SETFD(fcntl::FdFlag::FD_CLOEXEC)).unwrap();
    fcntl::fcntl(&re_write, fcntl::FcntlArg::F_SETFD(fcntl::FdFlag::FD_CLOEXEC)).unwrap();
    let (read_fd, write_fd) =  pipe().map_err(|error| status::ShellError::Pipe(error))?;
    let left_pid: Pid = match unsafe {fork()} {
        Ok(ForkResult::Child) => {
            drop(read_fd);
            drop(le_read);
            dup2_stdout(&write_fd).unwrap();
            drop(write_fd);
            unsafe {
                set_sig_to_def!();
            }
            setpgid(Pid::from_raw(0), Pid::from_raw(0)).unwrap();
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
    match unsafe {fork()} {
        Ok(ForkResult::Child) => {
            drop(write_fd);
            drop(re_read);
            dup2_stdin(&read_fd).unwrap();
            drop(read_fd);
            setpgid(Pid::from_raw(0), left_pid).unwrap();
            unsafe {
                set_sig_to_def!();
            }
            match execvp(&args2[0], args2) {
                Err(error) => {
                    let error = error as i32;
                    let error = error.to_ne_bytes();
                    write(&re_write, &error).unwrap();
                    exit(1);
                }
            }
        }
        Err(error) => return Err(status::ShellError::Fork(error)),
        _ => ()
    };
    drop(read_fd);
    drop(write_fd);
    drop(le_write);
    drop(re_write);
    let tty = open("/dev/tty", OFlag::O_RDWR, Mode::empty()).unwrap();
    tcsetpgrp(&tty, left_pid).unwrap();
    waitpid(Pid::from_raw(left_pid.as_raw() * -1), None).unwrap();
    let mut left_buff = [0u8; 4];
    let mut right_buff = [0u8; 4];
    tcsetpgrp(&tty, getpid()).unwrap();
    let lb = read(le_read, &mut left_buff).unwrap();
    let rb = read(re_read, &mut right_buff).unwrap();
    tcsetpgrp(&tty, left_pid).unwrap();
    if lb == 4 {
        let error = i32::from_ne_bytes(left_buff);
        let _ = killpg(left_pid, SIGTERM);
        tcsetpgrp(&tty, getpid()).unwrap();
        return Err(status::ShellError::Exec(Errno::from_raw(error)));
    }
    else if rb == 4 {
        let error = i32::from_ne_bytes(right_buff);
        let _ = killpg(left_pid, SIGTERM);
        tcsetpgrp(&tty, getpid()).unwrap();
        return Err(status::ShellError::Exec(Errno::from_raw(error)));
    }
    let res = waitpid(Pid::from_raw(left_pid.as_raw() * -1), None).unwrap();
    tcsetpgrp(&tty, getpid()).unwrap();
    dup2_stdout(saved_stdout).expect("couldn't restore stdout");
    dup2_stdin(saved_stdin).expect("couldn't restore stdin!");
    match res {
        WaitStatus::Exited(_, code) => Ok(status::Returns::Code(code)),
        WaitStatus::Signaled(_, sig, _) => Ok(status::Returns::ShellSignal(sig)),
        _ => Ok(status::Returns::Code(0)),
    }
}
fn exec_and(args1: &[CString], args2: &[CString]) -> status::ShellResult {
    let res = exec_extern(args1);
    if let Ok(status::Returns::Code(0)) = res {
        return exec_extern(args2);
    }
    res
}