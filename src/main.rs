pub extern crate regex;
pub extern crate json;
pub use regex::Regex;
use std::fs;

mod parser;
use parser::*;

fn print_info(exp :&SyntaxTree, indents :usize) {
    println!("Rule: \"{}\"", &exp.rule);
    for (k, v) in &exp.args {
        print!("{}-{}:{}: ", " ".repeat(indents*4), k, &v.1);
        match &v.0 {
            Expression::Ident(name) => println!("{}", name),
            Expression::Exp(child) => print_info(&child, indents+1)
        }
    }
}

fn main() {
    let content = fs::read("hello.eas").expect("Error!");
    let content = String::from_utf8(content).expect("Error!");

    let lang = &fs::read("language.json").expect("Error!");
    let mut lang = json::parse(std::str::from_utf8(&lang).expect("Error!")).unwrap();
    let tokenable = Regex::new(&jpath!(lang, tokenable).unwrap()).unwrap();

    find_template(&mut lang, &tokenable).unwrap();

    print_info(&first_phrase(&content, &lang["syntax"], &tokenable, "block").unwrap().1, 0);
}

#[macro_export]
macro_rules! jpath {
	($ln:expr, $firstname:meta $(.$names:meta)*) => {{
		$ln[stringify!($firstname).to_ascii_lowercase()]$([stringify!($names).to_ascii_lowercase()])*
		.as_str().ok_or(stringify!(Cannot found: $firstname$(.$names)*))
	}};
}