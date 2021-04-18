enum Expression {
    Ident(String), Exp(SyntaxTree)
}
struct SyntaxTree {
    rule: String,
    args: HashMap<String, (Expression, String)>
}