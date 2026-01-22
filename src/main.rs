use std::{io::{self, Write}, thread::sleep, time::Duration};

mod exec;
mod parse;
mod status; 
fn main() {
    loop {
        print!("> ");
        io::stdout().flush().unwrap();
        let input = parse::parse_input();
        let args = parse::split_input(&input);
        exec::execute(args).ok();
        sleep(Duration::from_millis(1));
    }
}
