use crate::status::*;
use nix::unistd;
use rustyline::DefaultEditor;
pub fn exit() -> Returns {
    Returns::ExitSig
}
pub fn version() -> Returns {
    println!("version: 0.2.3");
    Returns::Code(0)
}
pub fn cd(dir: &str) -> ShellResult {
    unistd::chdir(dir).map_err(|error| ShellError::IO(error))?;
    Ok(Returns::Code(0))
}
pub fn history(line_editor: &DefaultEditor) -> Returns {
    for (i, j) in line_editor.history().iter().enumerate() {
        println!("{}: {}", i+1, j);
    }
    Returns::Code(0)
}