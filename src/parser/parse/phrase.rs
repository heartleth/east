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
                    code_at = token_from(&rule_s.to_string(), ck, tokenable);
                    if rule_s[ck..code_at].trim() != "{" {
                        types.push(true);
                        tokens.push(rule_s[ck..code_at].trim());
                    }
                    else {
                        let ck = code_at;
                        for c in rule_s[code_at..].chars() {
                            code_at += 1;
                            if c == '}' {
                                tokens.push(&rule_s[ck..code_at-1]);
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
    let mut ret = SyntaxTree {
        rule: String::new(),
        args: HashMap::new()
    };
    let mut code_idx = 0;
    let mut end = 0;
    let mut candidates :Vec<(usize, &JsonValue, bool, HashMap<&str, (&str, (usize, usize))>, usize)> = Vec::new();
    let mut to_drop = Vec::new();
    let s_tos = &s.to_string();

    let mut i = 0;
    for ruleset in rules[token_type].members() {
        for rule in ruleset.members() {
            candidates.push((i, rule, false, HashMap::new(), 0));
            i += 1;
        }
    }
    let mut cck = 0;
    while code_idx < s.len() {
        to_drop.clear();
        let ck = code_idx;
        let info = token_from_s(s_tos, ck, tokenable);
        let token = &info.1.trim();
        code_idx = info.0;
        for rule in candidates.iter_mut() {
            let name = rule.1["name"].as_str().unwrap();
            let tokens = &rule.1["tokens"];
            let types :Vec<&json::JsonValue> = tokens.members().next().unwrap().members().collect();
            let tokens :Vec<&json::JsonValue> = tokens.members().last().unwrap().members().collect();
            if let Some(is_true) = types.get(rule.4) {
                let mut will_remain = true;
                if is_true.as_bool().ok_or("Error!")? {
                    if let Some(expected) = &tokens[rule.4].as_str() {
                        if expected != token {
                            to_drop.push(rule.0);
                        }
                        rule.4 += 1;
                        rule.2 = false;
                        will_remain = false;
                    }
                }
                else if let Some(will_expect) = types.get(rule.4+1) {
                    if will_expect.as_bool().unwrap() {
                        if let Some(expected) = &tokens[rule.4+1].as_str() {
                            if expected == token {
                                let token_id :Vec<&str> = tokens[rule.4].as_str().unwrap().split(":").collect();
                                let token_name = *token_id.first().ok_or("No token name!")?;
                                rule.3.insert(token_name, (token_type, (cck, ck)));
                                will_remain = false;
                                rule.2 = false;
                                rule.4 += 2;
                            }
                        }
                    }
                }
                
                if will_remain {
                    let token_id :Vec<&str> = tokens[rule.4].as_str().unwrap().split(":").collect();
                    let token_name = *token_id.first().ok_or("No token name!")?;
                    let token_type = *token_id.last().ok_or("No token type!")?;
                    if token_type == "ident" {
                        rule.3.insert(token_name, (token_type, (ck, code_idx)));
                        rule.4 += 1;
                    }
                    else {
                        if rule.2 {
                            (rule.3.get_mut(token_name).unwrap().1).1 = code_idx;
                        }
                        else {
                            rule.3.insert(token_name, (token_type, (ck, code_idx)));
                            rule.2 = true;
                        }
                    }
                }
            }
            else {
                to_drop.push(rule.0);
            }
        }

        candidates.retain(|x|!to_drop.contains(&x.0));
        
        if candidates.len() == 0 {
            end = ck;
            break;
        }
        cck = ck;
    }
    
    candidates.retain(|x| {
        let len = x.1["tokens"].members().last().unwrap().members().len();
        x.4 == len || (x.4 == len-1 && x.2)
    });

    if let Some(code) = candidates.first() {
        let name = code.1["name"].as_str().unwrap();
        end = s.len();
        ret.rule = format!("{}.{}", token_type, name);
        for (k, v) in &code.3 {
            if v.0 == "ident" {
                ret.args.insert(k.to_string(), (Expression::Ident(s[(v.1).0..(v.1).1].to_string()), v.0.to_string()));
            }
            else {
                ret.args.insert(k.to_string(), (Expression::Exp(first_phrase(&s[(v.1).0..(v.1).1].trim(), rules, tokenable, v.0)?.1), v.0.to_string()));
            }
        }
    }
    else {
        return Err("No rule matches.");
    }

    return Ok((end, ret));
}