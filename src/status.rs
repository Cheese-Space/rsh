use nix::sys::signal::Signal;
pub type ShellResult = Result<Returns, ShellError>;
pub enum ShellError {
    Fork,
    IO(nix::errno::Errno)
}
pub enum Returns {
    Code(Option<i32>),
    ShellSignal(Signal),
    ExitSig
}