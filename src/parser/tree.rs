use std::collections::HashMap;

pub enum Expression {
    Ident(String), Exp(SyntaxTree)
}
pub struct SyntaxTree {
    pub args: HashMap<String, (Expression, String)>,
    pub rule: String
}