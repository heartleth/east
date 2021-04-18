use std::collections::HashMap;
pub extern crate regex;
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
    let mut rules = HashMap::new();
    rules.insert("exp", vec![
        vec![ String::from("{a:ident}") ],
        vec![ String::from("({a:exp})") ],
        vec![ String::from("{obj:ident}.{function:ident}({args:exp-multi})") ],
        vec![ String::from("{function:ident}({args:exp-multi})") ],
        vec![
            String::from("{1:exp} << {2:exp}"),
            String::from("{1:exp} >> {2:exp}"),
        ],
        vec![
            String::from("{1:exp} - {2:exp}"),
            String::from("{1:exp} + {2:exp}"),
        ],
        vec![
            String::from("{1:exp} * {2:exp}"),
            String::from("{1:exp} / {2:exp}")
        ],
        vec![ String::from("{obj:ident}.{member:exp}") ],
        vec![ String::from("{context:ident}::{member:exp}") ],
    ]);
    rules.insert("exp-multi", vec![
        vec![ String::from("{a:exp}") ],
        vec![ String::from("{1:exp}, {2:exp-multi}") ],
    ]);
    rules.insert("type", vec![
        vec![ String::from("{t:ident}") ],
        vec![ String::from("const {t:type}") ],
        vec![ String::from("{t:type} const") ],
        vec![ String::from("{super:type}::{chile:type}") ],
        vec![ String::from("{t:ident}<{args:type-multi}>") ],
    ]);
    rules.insert("type-multi", vec![
        vec![ String::from("{t:type}") ],
        vec![ String::from("{a:exp}") ],
        vec![ String::from("{1:type}, {2:type-multi}") ],
        vec![ String::from("{1:exp}, {2:type-multi}") ],
    ]);
    rules.insert("statement", vec![
        vec![ String::from("{t:type} {name:ident};") ],
        vec![ String::from("{t:type} {name:ident} = {a:exp};") ],
        vec![ String::from("{a:exp};") ],
    ]);
    rules.insert("block", vec![
        vec![ String::from("{a:statement}") ],
        vec![ String::from("{1:statement}\n{2:block}") ],
        vec![ String::from("{1:statement}{2:block}") ],
    ]);
    let tokenable = Regex::new("^([{}]|[=;]|<<|>>|::|\\.|,|[()]|[<>+\\-*/]|\\+=|[0-9]+|[a-zA-Z_][a-zA-Z_0-9]*)$").expect("Error!");

    // println!("{}", token_from(&" {".to_string(), 0, &tokenable));
    print_info(&parse(&content, &rules, "statement", &tokenable).expect("Error!"), 0);
}