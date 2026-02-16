use std::ffi::CString;
use crate::status::ShellError;
pub fn split_input(input: &str) -> Result<Vec<CString>, ShellError> {
    let input = input.split_whitespace();
    let mut c_input: Vec<CString> = Vec::new();
    for i in input {
        let val = CString::new(i).map_err(|_| ShellError::CStringNullByte)?;
        c_input.push(val);
    }
    Ok(c_input)
}