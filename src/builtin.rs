use crate::status::Returns;
pub fn exit() -> Returns {
    Returns::ExitSig
}
pub fn version() -> Returns {
    println!("version: 0.1.0");
    Returns::Code(None)
}