use std::io;
use std::ffi::CString;
pub fn parse_input() -> String {
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input
} 
pub fn split_input(input: &str) -> Vec<CString> {
    input.split_whitespace().map(|i| CString::new(i).unwrap()).collect()
}