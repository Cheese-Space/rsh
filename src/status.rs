use nix::sys::signal::Signal;
pub type ShellResult = Result<Returns, ShellError>;
pub enum ShellError {
    Fork,
    Execv(nix::errno::Errno),
    IO
}
pub enum Returns {
    Code(Option<i32>),
    ShellSignal(Signal),
    ExitSig
}