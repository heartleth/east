pub extern crate handlebars;
pub extern crate regex;
pub extern crate json;
pub use regex::Regex;
use std::fs;

mod parser;
use parser::*;
mod assemble;

fn main() {
    use std::env;
    let content = fs::read(env::args().nth(1).unwrap()).expect("Error!");
    let content = String::from_utf8(content).expect("Error!");
    
    let target = fs::read(env::args().nth(3).unwrap()).expect("Error!");
    let target = json::parse(&String::from_utf8(target).expect("Error!")[..]).unwrap();

    let lang = &fs::read(env::args().nth(2).unwrap()).expect("Error!");
    let mut lang = json::parse(std::str::from_utf8(&lang).expect("Error!")).unwrap();
    let tokenable = Regex::new(&jpath!(lang, tokenable).unwrap()).unwrap();

    find_template(&mut lang, &tokenable).unwrap();

    let ast = &first_phrase(&content, &lang["syntax"], &tokenable, "root", false).unwrap();
    let mut renderer = handlebars::Handlebars::new();
    let mut lf_reg = std::collections::HashMap::new();
    renderer.register_escape_fn(|e|e.to_string());
    fs::write(format!("{}.{}", env::args().nth(4).unwrap(), target["ext"]), assemble::assemble(&ast.1, &target, &mut renderer, "root", &mut lf_reg).unwrap()).unwrap();
    for cmd in lang["then"].members() {
        if cfg!(windows) {
            std::process::Command::new("cmd")
                .args(&["/c", &cmd.as_str().unwrap()]).output()
                .expect("Error!");
        }
    }
}

#[macro_export]
macro_rules! jpath {
	($ln:expr, $firstname:meta $(.$names:meta)*) => {{
		$ln[stringify!($firstname).to_ascii_lowercase()]$([stringify!($names).to_ascii_lowercase()])*
		.as_str().ok_or(stringify!(Cannot found: $firstname$(.$names)*))
	}};
}