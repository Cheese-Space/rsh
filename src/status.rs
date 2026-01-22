use nix::sys::signal::Signal;
pub type ShellResult = Result<Returns, ShellError>;
pub enum ShellError {
    Fork,
    Execv,
    IO,
    NotFound
}
pub enum Returns {
    Code(i32),
    ShellSignal(Signal)
}