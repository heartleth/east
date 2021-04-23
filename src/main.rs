pub extern crate handlebars;
pub extern crate regex;
pub extern crate json;
pub use regex::Regex;
use std::fs;

mod parser;
use parser::*;
mod assemble;

// fn print_info(exp :&SyntaxTree, indents :usize) {
//     println!("Rule: \"{}\"", &exp.rule);
//     for (k, v) in &exp.args {
//         print!("{}-{}:{}: ", " ".repeat(indents*4), k, &v.1);
//         match &v.0 {
//             Expression::Ident(name) => println!("{}", name),
//             Expression::Exp(child) => print_info(&child, indents+1)
//         }
//     }
// }

fn main() {
    use std::env;
    let content = fs::read(env::args().nth(1).unwrap()).expect("Error!");
    let content = String::from_utf8(content).expect("Error!");
    
    let target = fs::read(env::args().nth(2).unwrap()).expect("Error!");
    let target = json::parse(&String::from_utf8(target).expect("Error!")[..]).unwrap();

    let lang = &fs::read("language.json").expect("Error!");
    let mut lang = json::parse(std::str::from_utf8(&lang).expect("Error!")).unwrap();
    let tokenable = Regex::new(&jpath!(lang, tokenable).unwrap()).unwrap();

    find_template(&mut lang, &tokenable).unwrap();

    let ast = &first_phrase(&content, &lang["syntax"], &tokenable, "root", false).unwrap();
    let mut renderer = handlebars::Handlebars::new();
    let mut lf_reg = std::collections::HashMap::new();
    renderer.register_escape_fn(|e|e.to_string());
    fs::write(format!("result.{}", target["ext"]), assemble::assemble(&ast.1, &target, &mut renderer, "root", &mut lf_reg).unwrap()).unwrap();
}

#[macro_export]
macro_rules! jpath {
	($ln:expr, $firstname:meta $(.$names:meta)*) => {{
		$ln[stringify!($firstname).to_ascii_lowercase()]$([stringify!($names).to_ascii_lowercase()])*
		.as_str().ok_or(stringify!(Cannot found: $firstname$(.$names)*))
	}};
}