use std::collections::HashMap;
use crate::jpath;
use regex::Regex;
use super::*;
use json::*;

pub fn token_from(s :&String, pos :usize, tokenable :&Regex)->usize {
    if s.len() == 0 { return 0; }
    let mut end = s.len();
    let mut checking = String::new();
    let c = s.chars().nth(pos).unwrap();
    if c != ' ' {
        if (s.chars().nth(pos+1) != Some('{') && s.chars().nth(pos+1) != Some('}')) || c != '\\' {
            checking.push(c);
        }
    }

    for i in pos+1 .. s.len() {
        let c = s.chars().nth(i).unwrap();
        if c != ' ' || i != s.len()-1 {
            if (s.chars().nth(i+1) != Some('{') && s.chars().nth(i+1) != Some('}')) || c != '\\' {
                checking.push(c);
                if !tokenable.is_match(&checking[..]) {
                    end = i;
                    break;
                }
            }
        }
    }
    end
}
pub fn token_from_s(s :&String, pos :usize, tokenable :&Regex)->String {
    let mut checking = String::new();
    let c = s.chars().nth(pos).unwrap();
    if c != ' ' {
        if (s.chars().nth(pos+1) != Some('{') && s.chars().nth(pos+1) != Some('}')) || c != '\\' {
            checking.push(c);
        }
    }

    for i in pos+1 .. s.len() {
        let c = s.chars().nth(i).unwrap();
        if c != ' ' || i != s.len()-1 {
            if (s.chars().nth(i+1) != Some('{') && s.chars().nth(i+1) != Some('}')) || c != '\\' {
                checking.push(c);
                if !tokenable.is_match(&checking[..]) {
                    checking.pop();
                    return checking;
                }
            }
        }
    }
    checking
}

pub fn token_from_reverse(s :&String, pos :usize, tokenable :&Regex)->usize {
    let mut end = 0;
    let mut checking = String::new();

    for i in (0 .. pos).rev() {
        let c = s.chars().nth(i).unwrap();
        if c != ' ' || (i != 1 && i != pos - 1) {
            if (s.chars().nth(i+1) != Some('{') && s.chars().nth(i+1) != Some('}')) || c != '\\' {
                checking.insert(0, c);
            }
        }
        if !tokenable.is_match(&checking[..]) {
            end = i + 1;
            break;
        }
    }
    end
}

pub fn parse(s :&String, rules :&JsonValue, token_type :&str, tokenable :&Regex)->std::result::Result<SyntaxTree, &'static str> {
    // println!("{}", s);
    let mut ret = SyntaxTree {
        rule: String::new(),
        args: HashMap::new()
    };
    let mut exists = false;
    let mut is_ok = false;
    let mut shortest = 9999;

    for ruleset in rules["syntax"][token_type].members() {
        for rule in ruleset.members() {
            let rulename = rule["name"].as_str().ok_or("No name field.")?;
            let rule = rule["rule"].as_str().ok_or("No rule field.")?;
            // println!("{}", rule);
            let mut temp = SyntaxTree {
                rule: String::new(),
                args: HashMap::new()
            };
            let args = &mut temp.args;
            let mut rule_idx = 0;
            let mut code_idx = 0;
            let mut first_arg_length = 0;
            
            let mut token = String::new();
            let mut in_token = false;
            let mut first_letter_token = true;
            let mut token_start = 0;
            let mut token_name = String::new();
            
            for c in rule.chars() {
                if c == '{' {
                    if rule_idx > 0 && rule.chars().nth(rule_idx-1).unwrap_or('?') == '\\' {
                        token.pop();
                        token.push(c);
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
                    else {
                        token_name = String::new();
                        token = String::new();
                        in_token = true;
                    }
                }
                else if c == '}' {
                    if rule_idx > 0 && rule.chars().nth(rule_idx-1).unwrap_or('?') == '\\' {
                        token.pop();
                        token.push(c);
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
                    else {
                        token = String::new();
                        if rule_idx == rule.len()-1 {
                            let inner = String::from(s[code_idx..].trim());
                            let token_id :Vec<&str> = token_name.split(":").collect();
                            let token_name = *token_id.first().ok_or("No token name!")?;
                            let token_type = *token_id.last().ok_or("No token type!")?;
                            let is_ident = token_type == "ident" && token_from(&inner, 0, tokenable)==inner.len();
                            if is_ident {
                                if first_arg_length != 0 {
                                    first_arg_length = inner.len();
                                }
                                args.insert(String::from(token_name), (Expression::Ident(inner), String::from(token_type)));
                            }
                            else {
                                let exp = parse(&inner, rules, &String::from(token_type), tokenable);
                                if let Ok(exp) = exp {
                                    args.insert(String::from(token_name), (Expression::Exp(exp), String::from(token_type)));
                                    if first_arg_length != 0 {
                                        first_arg_length = inner.len();
                                    }
                                }
                                else {
                                    is_ok = false;
                                    break;
                                }
                            }
                            is_ok = true;
                        }
                        else {
                            is_ok = false;
                            let token_id :Vec<&str> = token_name.split(":").collect();
                            let token_name = *token_id.first().ok_or("No token name!")?;
                            let token_type = *token_id.last().ok_or("No token type!")?;
                            let next_token = token_from_s(&rule.to_string(), rule_idx+1, tokenable);
                            if next_token == "{" { // TODO! Find better algorithm
                                if token_type == "ident" {
                                    let ck = code_idx;
                                    code_idx = token_from(s, code_idx, tokenable);
                                    args.insert(String::from(token_name), (Expression::Ident(String::from(s[ck..code_idx].trim())), String::from(token_type)));
                                    is_ok = true;
                                }
                                else {
                                    let token_start = code_idx;
                                    code_idx = s.len();
                                    while code_idx > 0 {
                                        let ck = code_idx;
                                        code_idx = token_from_reverse(s, code_idx, tokenable);
                                        if let Ok(inner) = parse(&String::from(s[token_start..code_idx].trim()), rules, token_type, tokenable) {
                                            if let Err(_) = parse(&String::from(s[token_start..ck].trim()), rules, token_type, tokenable) {
                                                args.insert(String::from(token_name), (Expression::Exp(inner), String::from(token_type)));
                                                is_ok = true;
                                                break;
                                            }
                                        }
                                    }
                                }
                            }
                            else {
                                let mut in_string = false;
                                let mut escaped = false;
                                let mut stack :Vec<char> = Vec::new();
                                token_start = code_idx;
                                while code_idx < s.len() {
                                    let ck = code_idx;
                                    code_idx = token_from(s, ck, tokenable);
                                    
                                    if stack.is_empty() && &s[ck..code_idx].trim() == &next_token {
                                        let inner = String::from(s[token_start..ck].trim());
                                        let is_ident = token_type == "ident" && token_from(&inner, 0, tokenable)==inner.len();
                                        if is_ident {
                                            if first_arg_length != 0 {
                                                first_arg_length = inner.len();
                                            }
                                            args.insert(String::from(token_name), (Expression::Ident(inner), String::from(token_type)));
                                        }
                                        else {
                                            let exp = parse(&inner, rules, &String::from(token_type), tokenable);
                                            if let Ok(exp) = exp {
                                                args.insert(String::from(token_name), (Expression::Exp(exp), String::from(token_type)));
                                                if first_arg_length != 0 {
                                                    first_arg_length = inner.len();
                                                }
                                            }
                                            else {
                                                is_ok = false;
                                                break;
                                            }
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
                                            _=>escaped=false
                                        };
                                    }
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
                if shortest > first_arg_length {
                    shortest = first_arg_length;
                    ret.rule = format!("{}.{}", token_type, rulename);
                    ret.args = temp.args;
                }
                exists = true;
            }
        }
        if exists {
            return Ok(ret);
        }
    }
    // println!("Type: {}, s: {}", token_type, s);
    Err("No rule matches.")
}