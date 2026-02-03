use serde::{Serialize, Deserialize};
use std::{fs, io::{self, BufRead, Write}};
use termion::color;
#[derive(Serialize, Deserialize)]
pub struct Conf {
    pub usercolor: String,
    pub errorcolor: String,
    pub separator: String
}
impl Conf {
    pub fn make_conf() {
        let mut ucolor = String::new();
        let mut ecolor = String::new();
        let mut separator = String::new();
        let mut buff = String::new();
        let mut stdin = io::stdin().lock();
        println!("rsh-configurator ver 0.1.0\nusercolor:\n0 [default]: light green\n1: light blue\n2: light cyan\n3: light yellow\n4: terminal default");
        stdin.read_line(&mut buff).unwrap();
        match buff.trim() {
            "0" => ucolor.push_str(&color::Fg(color::LightGreen).to_string()),
            "1" => ucolor.push_str(&color::Fg(color::LightBlue).to_string()),
            "2" => ucolor.push_str(&color::Fg(color::LightCyan).to_string()),
            "3" => ucolor.push_str(&color::Fg(color::LightYellow).to_string()),
            "4" => ucolor.push_str(&color::Fg(color::Reset).to_string()),
            _ => ucolor.push_str(&color::Fg(color::LightGreen).to_string())
        }
        buff.clear();
        println!("errorcolor:\n0: [default]: light red\n1: red\n2: light magenta\n3: magenta\n4: terminal default");
        stdin.read_line(&mut buff).unwrap();
        match buff.trim() {
            "0" => ecolor.push_str(&color::Fg(color::LightRed).to_string()),
            "1" => ecolor.push_str(&color::Fg(color::Red).to_string()),
            "2" => ecolor.push_str(&color::Fg(color::LightMagenta).to_string()),
            "3" => ecolor.push_str(&color::Fg(color::Magenta).to_string()),
            "4" => ecolor.push_str(&color::Fg(color::Reset).to_string()),
            _ => ecolor.push_str(&color::Fg(color::LightRed).to_string())
        }
        buff.clear();
        println!("separator:\n[default]: ->");
        stdin.read_line(&mut buff).unwrap();
        if buff.trim().is_empty() {
            separator.push_str("->");
        }
        else {
            separator.push_str(buff.trim());
        }
        let contents = Self {
            usercolor: ucolor,
            errorcolor: ecolor,
            separator
        };
        let contents = serde_json::to_string_pretty(&contents).unwrap();
        let mut file = fs::File::create("/usr/local/etc/rsh.json").unwrap();
        file.write_all(contents.as_bytes()).unwrap();
        println!("exit rsh for changes to take efect");
    }
    pub fn read_conf() -> Self {
        let file = match fs::read_to_string("/usr/local/etc/rsh.json") {
            Ok(f) => f,
            Err(error) => {
                eprintln!("error: {}\nusing defaults\ntip: you can make a conf using mkconf", error);
                return Self::default();
            }
        };
        let contents: Conf = match serde_json::from_str(&file) {
            Ok(c) => c,
            Err(error) => {
                eprintln!("error: {}\nusing defaults\ntip: you can make a conf using mkconf", error);
                return Self::default();
            }
        };
        contents
    }
}
impl Default for Conf {
    fn default() -> Self {
        Self {
            usercolor: color::Fg(color::LightGreen).to_string(),
            errorcolor: color::Fg(color::LightRed).to_string(),
            separator: "->".to_string()
        }
    }
}