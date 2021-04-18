use std::collections::HashMap;
pub extern crate regex;
pub use regex::Regex;
use std::fs;

mod parser;
use parser::*;

fn print_info(exp :&SyntaxTree, indents :usize) {
    println!("Rule: \"{}\"", &exp.rule);
    for (k, v) in &exp.args {
        print!("{}  {}:{}: ", " ".repeat(indents*4), k, &v.1);
        match &v.0 {
            Expression::Ident(name) => println!("{}", name),
            Expression::Exp(child) => print_info(&child, indents+1)
        }
    }
}

fn main() {
    let content = fs::read("hello.eas").expect("Error!");
    let content = String::from_utf8(content).expect("Error!");
    let mut rules = HashMap::new();
    rules.insert("exp", vec![
        vec![ String::from("{a:ident}") ],
        vec![ String::from("({a:exp})") ],
        vec![ String::from("{obj:ident}.{function:ident}({args:exp-multi})") ],
        vec![ String::from("{function:ident}({args:exp-multi})") ],
        vec![
            String::from("{1:exp} - {2:exp}"),
            String::from("{1:exp} + {2:exp}"),
        ],
        vec![
            String::from("{1:exp} * {2:exp}"),
            String::from("{1:exp} / {2:exp}")
        ],
    ]);
    rules.insert("exp-multi", vec![
        vec![ String::from("{a:exp}") ],
        vec![ String::from("{1:exp}, {2:exp-multi}") ],
    ]);
    let tokenable = Regex::new("^(\\.|,|[()]|[+\\-*/]|\\+=|[0-9]+|and|[a-zA-Z_][a-zA-Z_0-9]*)$").expect("Error!");

    print_info(&parse(&content, &rules, "exp", &tokenable).expect("Error!"), 0);
}