use nix::sys::signal::Signal;
pub type ShellResult = Result<Returns, ShellError>;
pub enum ShellError {
    Fork(nix::errno::Errno),
    IO(nix::errno::Errno),
    NoArg,
    CStringNullByte,
    DupStdout(nix::errno::Errno, String),
    DupStdin(nix::errno::Errno, String)
}
impl std::fmt::Display for ShellError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShellError::Fork(error) => write!(f, "error: {}", error.desc()),
            ShellError::IO(error) => write!(f, "error: {}", error.desc()),
            ShellError::NoArg => write!(f, "error: no argument(s) provided!"),
            ShellError::CStringNullByte => write!(f, "error: null byte found before end of string"),
            ShellError::DupStdout(error, filename) => write!(f, "error: {}\nfailed to redirect stdout to '{}'", error.desc(), filename),
            ShellError::DupStdin(error, filename) => write!(f, "error: {}\nfailed to redirect stdin to '{}'", error.desc(), filename)
        }
    }
}
pub enum Returns {
    Code(i32),
    ShellSignal(Signal),
    ExitSig
}