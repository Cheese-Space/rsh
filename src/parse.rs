use std::io;
use std::ffi::CString;
use crate::status::ShellError;
pub fn parse_input() -> String {
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input
} 
pub fn split_input(input: &str) -> Result<Vec<CString>, ShellError> {
    let input = input.split_whitespace();
    let mut c_input: Vec<CString> = Vec::new();
    for i in input {
        let val = match CString::new(i) {
            Ok(s) => s,
            Err(_) => return Err(ShellError::CStringNullByte)
        };
        c_input.push(val);
    }
    Ok(c_input)
}