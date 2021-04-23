use std::collections::HashMap;
use super::parser::SyntaxTree;
use handlebars::Handlebars;

pub fn assemble(ast :&SyntaxTree, lang :&json::JsonValue, renderer :&mut Handlebars, rule_type :&str, lf_reg :&mut HashMap<String, bool>)->std::result::Result<String, &'static str> {
    let type_id_raw = format!("{}-{}", &ast.rule, rule_type);
    let rule_name = &ast.rule;
    let rule_types = &lang["syntax"][rule_type];
    if !renderer.has_template(&type_id_raw) {
        for ruleset in rule_types.members() {
            for rule in ruleset.members() {
                if rule["name"] == rule_name[..] {
                    if let Some(template) = rule["rule"].as_str() {
                        lf_reg.insert(type_id_raw.to_string(), rule["lf"].as_bool().unwrap_or(false));
                        match renderer.register_template_string(&type_id_raw[..], template) {
                            Ok(_) => {},
                            Err(_) => { return Err("Render template Error!");}
                        };
                    }
                    else {
                        return Err("No `rule` in object.");
                    }
                }
            }
        }
    }
    
    use super::Expression::*;
    let mut args = HashMap::new();
    for (k, (v, t)) in &ast.args {
        if let Exp(exp) = v {
            args.insert(format!("{}-{}", k, t), assemble(&exp, lang, renderer, t, lf_reg)?);
        }
        else if let Ident(exp) = v {
            args.insert(format!("{}-{}", k, t), exp.to_string());
        }
    }
    let res = renderer.render(&type_id_raw, &args);
    match res {
        Ok(ret) => {
            if *lf_reg.get(&type_id_raw).unwrap() {
                Ok(ret + "\n")
            }
            else {
                Ok(ret)
            }
        },
        Err(err) => {
            println!("{:?}", err);
            return Err("Render Error!");
        }
    }
}