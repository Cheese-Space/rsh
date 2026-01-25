use crate::status::*;
use nix::unistd;
pub fn exit() -> Returns {
    Returns::ExitSig
}
pub fn version() -> Returns {
    println!("version: 0.1.0");
    Returns::Code(ReturnCode(0))
}
pub fn cd(dir: &str) -> ShellResult {
    match unistd::chdir(dir) {
        Ok(()) => Ok(Returns::Code(ReturnCode(0))),
        Err(error) => Err(ShellError::IO(error))
    }
}