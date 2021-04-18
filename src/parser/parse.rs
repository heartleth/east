use std::collections::HashMap;
extern crate regex;
use regex::Regex;

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

fn parse(s :&String, rules :&HashMap<&str, Vec<Vec<String>>>, token_type :&str, tokenable :&Regex)->Result<SyntaxTree, &'static str> {
    let mut ret = SyntaxTree {
        rule: String::new(),
        args: HashMap::new()
    };
    let mut exists = false;
    let mut is_ok = false;
    let mut shortest = 9999;

    for ruleset in rules.get(token_type).ok_or("No token type.")? {
        for rule in ruleset {
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
                if shortest > first_arg_length {
                    shortest = first_arg_length;
                    ret.rule = String::from(rule);
                    ret.args = temp.args;
                }
                exists = true;
            }
        }
        if exists {
            return Ok(ret);
        }
    }
    Err("No rule matches.")
}