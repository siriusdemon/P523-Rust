use std::vec::IntoIter;
use crate::syntax::*;

#[derive(Debug, Clone)]
pub struct Token {
    pub token: String,
    pub i: usize,
}


pub struct Scanner {
    expr: Vec<char>,
}


fn is_delimiter(c: char) -> bool {
    let delimiter = "(){}[]";
    match delimiter.find(c) {
        Some(_i) => true,
        None => false,
    } 
}

impl Scanner {
    pub fn new(expr: &str) -> Self {
        let expr: Vec<char> = expr.chars().collect();
        Self { expr }
    }

    pub fn scan(self) -> Vec<Token> {
        let mut res = vec![];
        let mut i = 0;
        while i < self.expr.len() {
            i = self.scan_expr(i, &mut res);
        }
        return res
    }

    pub fn scan_expr(&self, i: usize, tokens: &mut Vec<Token>) -> usize {
        // if i >= self.expr.len() { return i; }

        let c = self.expr[i];
        match c {
            cc if is_delimiter(cc) => {
                let tok = Token { token: format!("{}", cc), i };
                tokens.push(tok);
                i + 1
            }
            ' ' => i + 1,
            e => self.scan_sym(i, tokens),
        }
    }

    fn scan_sym(&self, i: usize, tokens: &mut Vec<Token>) -> usize {
        let mut sym = String::new();
        let mut j = i;
        while j < self.expr.len() && ! is_delimiter(self.expr[j]) && self.expr[j] != ' ' {
            sym.push(self.expr[j]);
            j = j + 1 
        }
        let tok = Token {token: sym, i};
        tokens.push(tok);
        return j;
    }
}




use Expr::*;
pub struct Parser {
    tokens: IntoIter<Token>,
    top: Option<Token>,
}

fn verify_symbol(sym: &str) -> bool {
    let noallow = "#'`,@~:[]{}()";
    for c in sym.chars() {
        if let Some(_idx) = noallow.find(c) {
            return false;
        }
    }
    return true;
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        let mut tokens = tokens.into_iter();
        let top = tokens.next();
        Self { tokens, top }
    }

    pub fn parse(mut self) -> Expr {
        self.parse_expr()
    }

    // at the very top level, only list and atom allowed
    pub fn parse_expr(&mut self) -> Expr {
        self.handle_newline();
        if let Some(ref t) = self.top() {
            if t.token.as_str() == "(" {
                return self.parse_list();
            }
            return self.parse_atom();
        }
        panic!("Unexpected Eof");
    }

    fn parse_atom(&mut self) -> Expr {
        let chars: Vec<char> = self.top().unwrap().token.chars().collect();
        match chars[0] {
            n if n.is_digit(10) => self.parse_integer(),
            e => self.parse_symbol(),
        }
    }

    fn parse_list(&mut self) -> Expr {
        let _left = self.remove_top();
        let top = self.top();
        match top.unwrap().token.as_str() {
            "begin" => self.parse_begin(),
            "set!" => self.parse_set(),
            "+" | "-" | "*" => self.parse_prim2(),
            e => panic!("Unexpected op: {}", e),
        }
    }


    fn parse_begin(&mut self) -> Expr {
        let _begin = self.remove_top();
        let mut begin_exprs = vec![];
        while let Some(ref t) = self.top() {
            if t.token.as_str() != ")" {
                let expr = self.parse_expr();
                begin_exprs.push(expr);
            } else {
                let _right = self.remove_top();
                return Expr::Begin(begin_exprs);
            }
        } 
        panic!("Parse Begin, unexpected eof");
    }

    fn parse_set(&mut self) -> Expr {
        let _set = self.remove_top();
        let e1 = self.parse_expr();
        let e2 = self.parse_expr();
        let _right = self.remove_top();
        Expr::Set(Box::new(e1), Box::new(e2))
    }

    fn parse_prim2(&mut self) -> Expr {
        let op = self.remove_top();
        let e1 = self.parse_expr();
        let e2 = self.parse_expr();
        let _right = self.remove_top();
        Expr::Prim2(op.unwrap().token, Box::new(e1), Box::new(e2))
    }

    fn parse_symbol(&mut self) -> Expr {
        let sym = self.remove_top().unwrap();
        if verify_symbol(&sym.token.as_str()) {
            return Expr::Symbol(sym.token);
        }
        panic!("Invalid Symbol {}", sym.token);
    }



    fn parse_integer(&mut self) -> Expr {
        let num = self.remove_top().unwrap();
        Expr::Int32 (num.token.parse().unwrap())
    }

    fn top(&self) -> Option<&Token> {
        self.top.as_ref()
    }

    fn remove_top(&mut self) -> Option<Token> {
        let new = self.tokens.next();
        match new {
            Some(t) => self.top.replace(t),
            None => self.top.take(),
        }
    }
    
    fn handle_newline(&mut self) {
        while self.top().unwrap().token.as_str() == "\n" {
            self.remove_top();
        }
    }
}

