use super::*;

pub fn find_template(rules :&mut JsonValue, tokenable :&Regex)->std::result::Result<(), &'static str> {
    for (_, typed) in rules["syntax"].entries_mut() {
        for ruleset in typed.members_mut() {
            for rule in ruleset.members_mut() {
                let rule_s = rule["rule"].as_str().ok_or("No rule field.")?;
                
                let mut token_at = 0;
                let mut code_at = 0;
                
                while code_at < rule_s.len() {
                    let ck = code_at;
                    code_at = token_from(&rule_s.to_string(), ck, tokenable);
                    if rule_s[ck..code_at].trim() != "{" {
                        break;
                    }
                    else {
                        for c in rule_s[code_at..].chars() {
                            code_at += 1;
                            if c == '}' {
                                break;
                            }
                        }
                    }
                    token_at += 1;
                }
                if code_at == rule_s.len() {
                    rule.insert("token_at", 9999).unwrap();
                }
                else {
                    rule.insert("token_at", token_at).unwrap();
                }
            }
        }
    }
    Ok(())
}