use std::io;
use std::ffi::CString;
use crate::status;
pub fn parse_input() -> String {
    // add error handeling, for now not important
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input
} 
pub fn split_input(input: &str) -> Vec<CString> {
    input.split_whitespace().map(|i| CString::new(i).unwrap()).collect()
}