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

pub fn first_phrase(s :&str, rules :&json::JsonValue, tokenable :&Regex, token_type :&str)->std::result::Result<(usize, SyntaxTree), &'static str> {
    // print!(" [[ {} :{} // ", s, token_type);
    let mut code_idx = 0;
    let mut end = 0;
    let mut candidates :Vec<(usize, &JsonValue, bool, HashMap<&str, (&str, (usize, usize))>, usize, bool, usize, &str, usize, usize)> = Vec::new();
    let mut to_drop = Vec::new();
    let s_tos = &s.to_string();

    let mut i = 0;
    let mut j = 0;
    for ruleset in rules[token_type].members() {
        for rule in ruleset.members() {
            candidates.push((i, rule, false, HashMap::new(), 0, true, 0, "", j, 9999));
            i += 1;
        }
        j += 1;
    }
    let mut cck = 0;
    while code_idx < s.len() {
        to_drop.clear();
        let ck = code_idx;
        let info = token_from_s(s_tos, ck, tokenable);
        let token = &info.1.trim();
        code_idx = info.0;
        
        print!("\n\n`{}`: ", token);
        for rule in candidates.iter_mut() {
            let name = &rule.1["name"].as_str().unwrap();
            let tokens = &rule.1["tokens"];
            let types :Vec<&json::JsonValue> = tokens.members().next().unwrap().members().collect();
            let tokens :Vec<&json::JsonValue> = tokens.members().last().unwrap().members().collect();
            
            print!(" {}[{}] ", name, rule.4);
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
                                if expected == token {
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
                                    // if token_type == "ident" && rule.9 == 0 {
                                    //     to_drop.push(rule.0);
                                    // }
                                }
                            }
                        }
                    }
                    
                    if will_remain {
                        let token_id :Vec<&str> = tokens[rule.4].as_str().unwrap().split(":").collect();
                        let token_name = *token_id.first().ok_or("No token name!")?;
                        let elem_token_type = *token_id.last().ok_or("No token type!")?;
                        rule.7 = token_name;

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
                            let inner = first_phrase(&s[ck..], rules, tokenable, elem_token_type)?;
                            rule.6 = ck + inner.0;
                            println!("{} {}", name, rule.6);
                            rule.3.insert(token_name, (elem_token_type, (ck, rule.6)));
                            rule.4 += 1;
                            rule.2 = true;
                        }
                    }
                }
                else {
                    to_drop.push(rule.0);
                }
            }
        }
        
        candidates.retain(|x|!to_drop.contains(&x.0));
        cck = ck;
    }
    
    candidates.retain(|x| {
        println!("\n{} {}", x.1["name"].as_str().unwrap(), token_type);
        let len = x.1["tokens"].members().last().unwrap().members().len();
        x.4 == len || (x.4 == len-1 && x.2)
    });
    
    if let Some(min) = &candidates.first() {
        let mut candidates_final = (end, *min, SyntaxTree {
            rule: String::new(),
            args: HashMap::new()
        });
        
        for code in &candidates {
            let mut ret = SyntaxTree {
                rule: String::new(),
                args: HashMap::new()
            };
            let mut is_skipping = false;
            let mut ret_args = HashMap::new();
            let name = code.1["name"].as_str().unwrap();
            
            for (k, v) in &code.3 {
                end = s.len();
                let token_type = v.0;
                if k == &code.7 {
                    let (a, _) = &v.1;
                    if token_type == "ident" {
                        end = a + token_from(&s[*a..].to_string(), 0, tokenable);
                        ret_args.insert(k.to_string(), (Expression::Ident(s[*a..end].to_string()), "ident".to_string()));
                    }
                    else {
                        let info = first_phrase(&s[*a..], rules, tokenable, token_type)?;
                        end = a + info.0;
                        ret_args.insert(k.to_string(), (Expression::Exp(info.1), token_type.to_string()));
                    }
                }
                else {
                    let (a, b) = v.1;
                    let s =  &s[a..b];
                    if token_type == "ident" {
                        let info = token_from(&s[a..b].trim().to_string(), 0, tokenable);
                        if info != s.trim().len() {
                            is_skipping = true;
                        }
                        ret_args.insert(k.to_string(), (Expression::Ident(s[a..b].to_string()), "ident".to_string()));
                    }
                    else {
                        let info = first_phrase(&s[a..b].trim(), rules, tokenable, token_type).unwrap();
                        if info.0 != s.trim().len() {
                            is_skipping = true;
                        }
                        ret_args.insert(k.to_string(), (Expression::Exp(info.1), token_type.to_string()));
                    }
                }
            }
            if !is_skipping {
                ret.rule = name.to_string();
                ret.args = ret_args;
                // if end > candidates_final.0 {
                //     candidates_final.0 = end;
                //     candidates_final.1 = code;
                //     candidates_final.2 = ret;
                // }
                if code.9 <= (candidates_final.1).9 && (candidates_final.1).8 == code.8 {
                    candidates_final.0 = end;
                    candidates_final.1 = code;
                    candidates_final.2 = ret;
                }
            }
        }
        // print!(" >> {} {} ]] ", (candidates_final.1).1["name"].as_str().unwrap(), candidates_final.0);
        return Ok((candidates_final.0, candidates_final.2));
    }
    
    println!("{}", token_type);
    return Err("No rule matches!");
}