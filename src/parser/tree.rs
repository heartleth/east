use std::collections::BTreeMap;

pub enum Expression {
    Ident(String), Exp(SyntaxTree)
}

pub struct SyntaxTree {
    pub args: BTreeMap<String, (Expression, String)>,
    pub rule: String
}