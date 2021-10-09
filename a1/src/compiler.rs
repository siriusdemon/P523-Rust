use crate::syntax::Expr;
use crate::parser::{Scanner, Parser};

pub struct ParsePass {}
impl ParsePass {
    pub fn run(self, expr: &str) -> Expr {
        let scanner = Scanner::new(expr);
        let tokens = scanner.scan();
        let parser = Parser::new(tokens);
        let expr = parser.parse();
        return expr;
    }
}



pub fn compile(s: &str) {
    let expr = ParsePass{}.run(s);
    println!("{}", expr);
}