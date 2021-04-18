use std::collections::BTreeMap;
extern crate regex;
use regex::Regex;
// use std::fs;

enum Expression {
    Ident(String), Exp(SyntaxTree)
}
struct SyntaxTree {
    rule: usize,
    args: BTreeMap<String, Expression>
}

fn token_from(s :&String, pos :usize, tokenable :&Regex)->usize {
    let mut end = s.len();
    let mut checking = String::new();
    let c = s.chars().nth(pos).unwrap();
    if c != ' ' {
        checking.push(c);
    }

    for i in pos+1 .. s.len() {
        let c = s.chars().nth(i).unwrap();
        if c != ' ' || i != s.len()-1 {
            checking.push(c);
        }
        if !tokenable.is_match(&checking[..]) {
            end = i;
            break;
        }
    }
    end
}

fn parse(s :&String, rules :&Vec<String>, tokenable :&Regex)->Result<SyntaxTree, &'static str> {
    let mut ret = SyntaxTree {
        rule: 0,
        args: BTreeMap::new()
    };
    let mut is_ok = false;
    let mut nth_rule = 0;
    for rule in rules {
        let args = &mut ret.args;
        let mut rule_idx = 0;
        let mut code_idx = 0;
        
        let mut token = String::new();
        let mut in_token = false;
        let mut first_letter_token = true;
        let mut next_token :&str;
        let mut token_start = 0;
        let mut token_name = String::new();
        
        for c in rule.chars() {
            if c == '{' {
                token_name = String::new();
                token = String::new();
                in_token = true;
            }
            else if c == '}' {
                token = String::new();
                if rule_idx == rule.len()-1 {
                    let inner = String::from(s[code_idx..].trim());
                    let is_ident = token_from(&inner, 0, tokenable)==inner.len();
                    if is_ident {
                        args.insert(String::from(&token_name), Expression::Ident(inner));
                    }
                    else {
                        let exp = parse(&inner, rules, tokenable)?;
                        args.insert(String::from(&token_name), Expression::Exp(exp));
                    }
                    is_ok = true;
                }
                else {
                    is_ok = false;
                    let mut in_string = false;
                    let mut escaped = false;
                    let mut stack :Vec<char> = Vec::new();
                    next_token = &rule[rule_idx+1..token_from(rule, rule_idx+1, tokenable)].trim();
                    token_start = code_idx;
                    while code_idx < s.len() {
                        let ck = code_idx;
                        code_idx = token_from(s, ck, tokenable);
                        if stack.is_empty() && &s[ck..code_idx].trim() == &next_token {
                            let inner = String::from(s[token_start..ck].trim());
                            let is_ident = token_from(&inner, 0, tokenable)==inner.len();
                            if is_ident {
                                args.insert(String::from(&token_name), Expression::Ident(inner));
                            }
                            else {
                                let exp = parse(&inner, rules, tokenable)?;
                                args.insert(String::from(&token_name), Expression::Exp(exp));
                            }
                            is_ok = true;
                            code_idx = ck;
                            break;
                        }
                        for elem in s[ck..code_idx].chars() {
                            match elem { // From Enpp-rust
                                '\\' => { escaped = in_string && !escaped },
                                '"' => { if !escaped { in_string = !in_string; } escaped=false; },
                                '(' => if !in_string { stack.push('(') },
                                ')' => if !in_string {
                                    if stack.is_empty() { return Err("Parentheses do not match."); }
                                    else if *stack.last().unwrap() == '(' { stack.pop(); }
                                    else { return Err("Parentheses do not match."); }
                                },
                                '{' => if !in_string { stack.push('{') },
                                '}' => if !in_string {
                                    if stack.is_empty() { return Err("Parentheses do not match."); }
                                    else if *stack.last().unwrap() == '{' { stack.pop(); }
                                    else { return Err("Parentheses do not match."); }
                                },
                                _=>{}
                            };
                        }
                    }
                    if !is_ok {
                        break;
                    }
                }
                in_token = false;
                first_letter_token = true;
                token_start = rule_idx + 1;
            }
            else if in_token {
                token_name.push(c);
            }
            else {
                if c != ' ' {
                    first_letter_token = false;
                    token.push(c);
                }
                else if !first_letter_token {
                    token.push(c);
                }
                else {
                    first_letter_token = true;
                    token_start += 1;
                }
                if rule_idx == rule.len()-1 || (tokenable.is_match(&token[..]) && !tokenable.is_match(&rule[token_start..=rule_idx+1])) {
                    let ck = code_idx;
                    code_idx = token_from(&s, code_idx, tokenable);
                    if s[ck..code_idx].trim() == token.trim() {
                        first_letter_token = true;
                        token_start = rule_idx + 1;
                        if rule_idx == rule.len()-1 {
                            is_ok = code_idx == s.len();
                        }
                    }
                    else {
                        break;
                    }
                    token = String::new();
                }
            }
            rule_idx += 1;
        }
        if is_ok {
            ret.rule = nth_rule;
            return Ok(ret);
        }
        nth_rule += 1;
    }
    Err("No rule matches.")
}

fn print_info(exp :&SyntaxTree, rules :&Vec<String>, indents :usize) {
    println!("Rule: \"{}\"", &rules[exp.rule]);
    for (k, v) in &exp.args {
        print!("{}  {}: ", " ".repeat(indents*4), k);
        match v {
            Expression::Ident(name) => println!("{}", name),
            Expression::Exp(child) => print_info(child, rules, indents+1)
        }
    }
}

fn main() {
    // let content = fs::read("hello.eas").expect("Error!");
    // let content = String::from_utf8(content).expect("Error!");
    let content = String::from("1 - 2 + 3");
    let rules = vec![
        String::from("{1} - {2}"),
        String::from("{1} + {2}"),
        String::from("{1} * {2}"),
        String::from("{1} / {2}"),
        String::from("({a})"),
    ];
    let tokenable = Regex::new("^([()]|[+\\-*/]|\\+=|[0-9]+|and|[a-zA-Z_][a-zA-Z_0-9]*)$").expect("Error!");

    print_info(&parse(&content, &rules, &tokenable).expect("Error!"), &rules, 0);
    // &parse(&content, &rules, &tokenable).expect("Error!");
}