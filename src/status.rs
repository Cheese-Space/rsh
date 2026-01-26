use nix::sys::signal::Signal;
use termion::color;
pub type ShellResult = Result<Returns, ShellError>;
pub enum ShellError {
    Fork,
    IO(nix::errno::Errno),
}
impl std::fmt::Display for ShellError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShellError::Fork => todo!(),
            ShellError::IO(error) => write!(f, "error: {}", error.desc()),
        }
    }
}
pub enum Returns {
    Code(ReturnCode),
    ShellSignal(Signal),
    ExitSig
}
pub struct ReturnCode(pub i32);
impl std::fmt::Display for ReturnCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReturnCode(0) => write!(f, "-> "),
            ReturnCode(failure) => write!(f, "[{}{}{}] -> ", color::Fg(color::LightRed), failure, color::Fg(color::Reset))
        }
    }
}