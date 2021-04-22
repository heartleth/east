use super::*;

pub fn find_template(rules :&mut json::JsonValue, tokenable :&Regex)->std::result::Result<(), &'static str> {
    for (_, typed) in rules["syntax"].entries_mut() {
        for ruleset in typed.members_mut() {
            for rule in ruleset.members_mut() {
                let mut types = Vec::new();
                let mut tokens = Vec::new();

                let rule_s = rule["rule"].as_str().ok_or("No rule field.")?;
                let mut code_at = 0;
                
                while code_at < rule_s.len() {
                    let ck = code_at;
                    let info = token_from_s(&rule_s.to_string(), ck, tokenable);
                    code_at = info.0;
                    let token = info.1;
                    if rule_s[ck..code_at].trim() != "{" || (ck > 0 && rule_s.chars().nth(ck-1) == Some('\\')) {
                        if rule_s[ck..code_at].trim() == "\\" && (rule_s.chars().nth(code_at) == Some('{') || rule_s.chars().nth(code_at) == Some('}')) {
                            
                        }
                        else {
                            types.push(true);
                            tokens.push(token);
                        }
                    }
                    else {
                        let ck = code_at;
                        for c in rule_s[code_at..].chars() {
                            code_at += 1;
                            if c == '}' {
                                tokens.push(rule_s[ck..code_at-1].to_string());
                                types.push(false);
                                break;
                            }
                        }
                    }
                }
                
                let mut info = json::Array::new();
                info.push(json::JsonValue::Array(types.into_iter().map(|x|json::JsonValue::Boolean(x)).collect()));
                info.push(json::JsonValue::Array(tokens.into_iter().map(|x|json::JsonValue::String(x.to_string())).collect()));
                rule.insert("tokens", info).unwrap();
            }
        }
    }
    Ok(())
}

pub fn first_phrase(s :&str, rules :&json::JsonValue, tokenable :&Regex, token_type :&str, notree :bool)->std::result::Result<(usize, SyntaxTree), &'static str> {
    let mut code_idx = 0;
    let mut is_going_out = false;
    let mut end;
    let mut candidates :Vec<(usize, &JsonValue, bool, BTreeMap<&str, (&str, (usize, usize))>, usize, bool, usize, &str, usize, usize, &str)> = Vec::new();
    let mut to_drop = Vec::new();
    let s_tos = &s.to_string();

    let mut i = 0;
    let mut j = 0;
    for ruleset in rules[token_type].members() {
        for rule in ruleset.members() {
            candidates.push((i, rule, false, BTreeMap::new(), 0, true, 0, "", j, 9999, ""));
            i += 1;
        }
        j += 1;
    }
    let mut cck = 0;
    let mut in_string = false;
    let mut escaped = false;
    let mut stack :Vec<char> = Vec::new();

    while code_idx < s.len() {
        to_drop.clear();
        let ck = code_idx;
        let info = token_from_s(s_tos, ck, tokenable);
        let token = &info.1.trim();
        code_idx = info.0;
        for rule in candidates.iter_mut() {
            let tokens = &rule.1["tokens"];
            let types :Vec<&json::JsonValue> = tokens.members().next().unwrap().members().collect();
            let tokens :Vec<&json::JsonValue> = tokens.members().last().unwrap().members().collect();
            if rule.6 <= ck {
                if let Some(is_true) = types.get(rule.4) {
                    let mut will_remain = true;
                    if is_true.as_bool().ok_or("Error!")? {
                        if let Some(expected) = &tokens[rule.4].as_str() {
                            if expected != token {
                                to_drop.push(rule.0);
                            }
                            rule.4 += 1;
                            rule.6 = code_idx;
                            rule.2 = false;
                            will_remain = false;
                            rule.5 = false;
                        }
                    }
                    else if let Some(will_expect) = types.get(rule.4+1) {
                        if will_expect.as_bool().unwrap() {
                            if let Some(expected) = &tokens[rule.4+1].as_str() {
                                if rule.5 && expected == token && (stack.is_empty() || (stack.len()==1 && (expected == &")" || expected == &"}"))) {
                                    let token_id :Vec<&str> = tokens[rule.4].as_str().unwrap().split(":").collect();
                                    let token_name = *token_id.first().ok_or("No token name!")?;
                                    let token_type = *token_id.last().ok_or("No token type!")?;
                                    if rule.2 {
                                        (rule.3.get_mut(token_name).unwrap().1).1 = ck;
                                    }
                                    else {
                                        rule.3.insert(token_name, (token_type, (cck, ck)));
                                    }
                                    will_remain = false;
                                    rule.2 = false;
                                    rule.4 += 2;
                                    rule.5 = false;
                                    let (a, b) = rule.3.get_mut(token_name).unwrap().1;
                                    rule.9 = b - a;
                                    rule.10 = token_name;
                                    if token_type == "ident" && rule.9 == 0 {
                                        to_drop.push(rule.0);
                                    }
                                    rule.6 = code_idx;
                                }
                            }
                        }
                    }
                    
                    if will_remain {
                        let token_id :Vec<&str> = tokens[rule.4].as_str().unwrap().split(":").collect();
                        let token_name = *token_id.first().ok_or("No token name!")?;
                        let elem_token_type = *token_id.last().ok_or("No token type!")?;
                        rule.7 = token_name;
                        rule.6 = code_idx;
                        
                        if elem_token_type == "ident" {
                            rule.4 += 1;
                            rule.3.insert(token_name, (elem_token_type, (ck, code_idx)));
                            rule.5 = false;
                            rule.2 = false;
                            rule.6 = code_idx;
                        }
                        else {
                            if rule.5 {
                                if elem_token_type != token_type && elem_token_type != "ident" {
                                    rule.5 = false;
                                }
                            }
                            if rule.5 {
                                if rule.2 {
                                    (rule.3.get_mut(token_name).unwrap().1).1 = code_idx;
                                }
                                else {
                                    rule.3.insert(token_name, (elem_token_type, (ck, code_idx)));
                                    rule.2 = true;
                                }
                            }
                            else {
                                let inner = first_phrase(&s[ck..], rules, tokenable, elem_token_type, true);
                                if let Ok(inner) = inner {
                                    if rule.1["name"] == "block" {
                                        println!("{} {}", s, inner.0);
                                    }
                                    rule.6 = ck + inner.0;
                                    rule.3.insert(token_name, (elem_token_type, (ck, rule.6)));
                                    rule.4 += 1;
                                    rule.5 = false;
                                    rule.2 = true;
                                }
                                else {
                                    to_drop.push(rule.0);
                                }
                            }
                        }
                    }
                }
                else {
                    let len = types.len();
                    if rule.4 < len {
                        to_drop.push(rule.0);
                    }
                }
            }

        }
        for elem in token.chars() {
            match elem { // From Enpp-rust
                '\\' => { escaped = in_string && !escaped },
                '"' => { if !escaped { in_string = !in_string; } escaped=false; },
                '(' => if !in_string { stack.push('(') },
                ')' => if !in_string {
                    if stack.is_empty() { is_going_out = true; break; }
                    else if *stack.last().unwrap() == '(' { stack.pop(); }
                    else { return Err("Parentheses do not match."); }
                },
                '{' => if !in_string { stack.push('{') },
                '}' => if !in_string {
                    if stack.is_empty() { is_going_out = true; break; }
                    else if *stack.last().unwrap() == '{' { stack.pop(); }
                    else { return Err("Parentheses do not match."); }
                },
                _=>escaped=false
            };
        }
        candidates.retain(|x|!to_drop.contains(&x.0));
        cck = ck;
        if is_going_out {
            break;
        }
    }
    
    candidates.retain(|x| {
        let len = x.1["tokens"].members().last().unwrap().members().len();
        x.4 == len
    });
    
    if let Some(min) = &candidates.first() {
        let mut candidates_final = (0, *min, SyntaxTree {
            rule: String::new(),
            args: BTreeMap::new()
        });
        
        for code in &candidates {
            end = code.6;
            let mut ret = SyntaxTree {
                rule: String::new(),
                args: BTreeMap::new()
            };
            let mut is_skipping = false;
            let mut ret_args = BTreeMap::new();
            let name = code.1["name"].as_str().unwrap();

            for (k, v) in &code.3 {
                let token_type = v.0;
                if k == &code.7 && code.2 && code.5 {
                    let (a, _) = &v.1;
                    if token_type == "ident" {
                        end = a + token_from(&s[*a..], 0, tokenable);
                        ret_args.insert(k.to_string(), (Expression::Ident(s[*a..end].to_string()), "ident".to_string()));
                    }
                    else {
                        let info = first_phrase(&s[*a..], rules, tokenable, token_type, notree);
                        if let Ok(info) = info {
                            end = a + info.0;
                            if !notree {
                                ret_args.insert(k.to_string(), (Expression::Exp(info.1), token_type.to_string()));
                            }
                        }
                        else {
                            is_skipping = true;
                        }
                    }
                }
                else if k == &code.10 || !notree {
                    let (a, b) = v.1;
                    let s =  &s[a..b];
                    if token_type == "ident" {
                        let info = token_from(s, 0, tokenable);
                        if info != s.len() {
                            is_skipping = true;
                        }
                        if !notree {
                            ret_args.insert(k.to_string(), (Expression::Ident(s.to_string()), "ident".to_string()));
                        }
                    }
                    else {
                        let info = first_phrase(&s, rules, tokenable, token_type, notree);
                        if let Ok(info) = info {
                            if info.0 != s.len() {
                                is_skipping = true;
                            }
                            if !notree {
                                ret_args.insert(k.to_string(), (Expression::Exp(info.1), token_type.to_string()));
                            }
                        }
                        else {
                            is_skipping = true;
                        }
                    }
                }
            }
            if !is_skipping {
                ret.rule = name.to_string();
                ret.args = ret_args;
                
                if end > candidates_final.0 {
                    candidates_final.0 = end;
                    if !notree {
                        candidates_final.1 = code;
                        candidates_final.2 = ret;
                    }
                }
                else if end == candidates_final.0 && (code.9 <= (candidates_final.1).9 && (candidates_final.1).8 == code.8) {
                    candidates_final.0 = end;
                    if !notree {
                        candidates_final.1 = code;
                        candidates_final.2 = ret;
                    }
                }
            }
        }
        return Ok((candidates_final.0, candidates_final.2));
    }
    return Err("No rule matches!");
}