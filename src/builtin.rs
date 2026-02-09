use crate::status::*;
use nix::unistd;
pub fn exit() -> Returns {
    Returns::ExitSig
}
pub fn version() -> Returns {
    println!("version: 0.2.1");
    Returns::Code(0)
}
pub fn cd(dir: &str) -> ShellResult {
    unistd::chdir(dir).map_err(|error| ShellError::IO(error))?;
    Ok(Returns::Code(0))
}