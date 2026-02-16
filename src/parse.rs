use std::ffi::CString;
use crate::status::ShellError;
#[macro_export]
macro_rules! get_input {
   ($conf:expr, $username:expr, $hostname:expr, $signal:expr, $return_code:expr, $rl:expr) => {{
       let readline: Result<String, ReadlineError>;
       if let Some(sig) = $signal {
           if sig == Signal::SIGINT {
               println!();
           }
           readline = $rl.readline(&format!("{}{}{}@{} [{}{}{}] {} ", $conf.usercolor, $username, color::Fg(color::Reset), $hostname, $conf.errorcolor, sig, color::Fg(color::Reset), $conf.separator));
       }
       else if  $return_code == 0 {
           readline = $rl.readline(&format!("{}{}{}@{} {} ", $conf.usercolor, $username, color::Fg(color::Reset), $hostname, $conf.separator));
       }
       else {
           readline = $rl.readline(&format!("{}{}{}@{} [{}{}{}] {} ", $conf.usercolor, $username, color::Fg(color::Reset), $hostname, $conf.errorcolor, $return_code, color::Fg(color::Reset), $conf.separator));
       }
       let input = match readline {
           Ok(s) => {
               $rl.add_history_entry(s.as_str()).unwrap();
               s
           }
           Err(ReadlineError::Interrupted) => continue,
           Err(ReadlineError::Eof) => {
               eprintln!("reached EOF"); 
               break;
           }
           Err(err) => {
               eprintln!("{:?}", err); 
               continue;
           }
       };
       input
   }};
}
pub fn split_input(input: &str) -> Result<Vec<CString>, ShellError> {
    let input = input.split_whitespace();
    let mut c_input: Vec<CString> = Vec::new();
    for i in input {
        let val = CString::new(i).map_err(|_| ShellError::CStringNullByte)?;
        c_input.push(val);
    }
    Ok(c_input)
}