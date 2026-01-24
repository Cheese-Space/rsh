use crate::status::{Returns, ShellError, ShellResult};
use nix::unistd;
pub fn exit() -> Returns {
    Returns::ExitSig
}
pub fn version() -> Returns {
    println!("version: 0.1.0");
    Returns::Code(None)
}
pub fn cd(dir: &str) -> ShellResult {
    match unistd::chdir(dir) {
        Ok(()) => Ok(Returns::Code(None)),
        Err(error) => Err(ShellError::IO(error))
    }
}