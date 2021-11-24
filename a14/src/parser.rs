use std::vec::IntoIter;
use std::collections::HashMap;
use std::collections::HashSet;

use crate::syntax::Scheme;
use crate::compiler::{gen_uvar, prim2_scm, prim1_scm, prim3_scm, quote_scm, let_scm};
use Scheme::*;

#[derive(Debug, Clone)]
pub struct Token {
    pub token: String,
    pub i: usize,
    pub line: usize,
    pub col: usize,
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


fn is_sym_terminal(c: char) -> bool {
    c == ';' || c == ' ' || c == '\n' || is_delimiter(c)
}


impl Scanner {
    pub fn new(expr: &str) -> Self {
        let expr: Vec<char> = expr.chars().collect();
        Self { expr }
    }

    pub fn scan(self) -> Vec<Token> {
        let mut res = vec![];
        let mut i = 0;
        let mut line = 1;
        let mut col = 1;
        while i < self.expr.len() {
            i = self.scan_expr(i, &mut line, &mut col, &mut res);
        }
        return res
    }

    pub fn scan_expr(&self, mut i: usize, line: &mut usize, col: &mut usize, tokens: &mut Vec<Token>) -> usize {
        let c = self.expr[i];
        match c {
            cc if is_delimiter(cc) => {
                let tok = Token { token: format!("{}", cc), i, line: *line, col: *col };
                tokens.push(tok);
                *col = *col + 1;
                i + 1
            }
            ' ' => {
                *col += 1;
                i + 1
            }
            '\n' => {
                *col = 0;
                *line += 1;
                i + 1
            }
            ';' => {
                i = i + 1;  // skip ;
                while i < self.expr.len() && self.expr[i] != '\n' {
                    i = i + 1;
                }
                *col = 0;
                *line += 1;
                return i;
            }
            '\'' | '#' => {
                let tok = Token { token: format!("{}", c), i, line: *line, col: *col };
                tokens.push(tok);
                *col = *col + 1;
                i = i + 1;
                assert!(self.expr[i] != ' ', "Invalid quote/hash in line {}, col {}", line, col);       // make sure no space following quote
                return i;
            }
            e => self.scan_sym(i, line, col, tokens),
        }
    }

    fn scan_atom<F>(&self, i: usize, line: &mut usize, col: &mut usize, tokens: &mut Vec<Token>, mut terminal: F) -> usize 
        where F: FnMut (char) -> bool  {
        let mut sym = String::new();
        let mut j = i;
        while j < self.expr.len() && !terminal(self.expr[j]) {
            sym.push(self.expr[j]);
            j = j + 1;
        }
        let tok = Token {token: sym, i, line: *line, col: *col};
        *col += tok.token.len();
        tokens.push(tok);
        return j;
    }

    fn scan_sym(&self, i: usize, line: &mut usize, col: &mut usize, tokens: &mut Vec<Token>) -> usize {
        return self.scan_atom(i, line, col, tokens, is_sym_terminal);
    }
}




use Scheme::*;
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

fn is_pair(left: &str, right: &str) -> bool {
    (left == "(" && right == ")") || 
    (left == "[" && right == "]") || 
    (left == "{" && right == "}")
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        let mut tokens = tokens.into_iter();
        let top = tokens.next();
        Self { tokens, top }
    }

    pub fn parse(mut self) -> Scheme {
        self.parse_expr()
    }

    pub fn parse_expr(&mut self) -> Scheme {
        if let Some(ref t) = self.top() {
            if t.token.as_str() == "(" || t.token.as_str() == "[" {
                return self.parse_list();
            }
            return self.parse_atom();
        }
        panic!("Unexpected Eof");
    }

    fn parse_list(&mut self) -> Scheme {
        let _left = self.remove_top();
        let top = self.top().unwrap().token.as_str();
        match top {
            "letrec" => self.parse_letrec(),
            "lambda" => self.parse_lambda(),
            "begin" => self.parse_begin(),
            "set!" => self.parse_set(),
            "if" => self.parse_if(),
            "let" => self.parse_let(),
            "car" | "cdr" | "make-vector" | "vector-length" | "procedure?" |
            "boolean?" | "fixnum?" | "null?" | "pair?" | "vector?"
                => self.parse_prim1(),
            "+" | "-" | "*" | "logor" | "logand" | "sra" |
            "=" | ">" | "<" | ">=" | "<=" | "eq?" |
            "cons" | "vector-ref" | "set-car!" | "set-cdr!"
                => self.parse_prim2(),
            "vector-set!" => self.parse_prim3(),
            "nop" => self.parse_nop(),
            "void" => self.parse_void(),
            "true" | "false" => self.parse_bool(),
            s_expr => self.parse_funcall(),
        }
    }

    fn parse_letrec(&mut self) -> Scheme {
        let _letrec = self.remove_top();
        let _lambda_left = self.remove_top();
        let mut bindings = HashMap::new();
        while let Some(ref t) = self.top() {
            if t.token.as_str() != ")" {
                let (k, lambda) = self.parse_binding();
                bindings.insert(k, lambda);
            } else {
                let _lambda_right = self.remove_top();
                let body = self.parse_expr();
                let _letrec_right = self.remove_top();
                return Scheme::Letrec(bindings, Box::new(body));
            }
        }
        panic!("Parse letrec, unexpected eof");
    }

    fn parse_lambda(&mut self) -> Scheme {
        let _lambda = self.remove_top().unwrap();
        let _args_left = self.remove_top();
        let mut args = vec![];
        while let Some(ref t) = self.top() {
            if t.token.as_str() != ")" {
                let arg = self.remove_top().unwrap().token;
                args.push(arg);
            } else {
                let _args_right = self.remove_top();
                let tail = self.parse_expr();
                let _right = self.remove_top();
                return Scheme::Lambda(args, Box::new(tail));
            }
        } 
        panic!("Parse Lambda, unexpected eof");
    }


    fn parse_begin(&mut self) -> Scheme {
        let _begin = self.remove_top();
        let mut begin_exprs = vec![];
        while let Some(ref t) = self.top() {
            if t.token.as_str() != ")" {
                let expr = self.parse_expr();
                begin_exprs.push(expr);
            } else {
                assert!(begin_exprs.len() > 0, "begin expr is empty!");
                let _right = self.remove_top();
                return Scheme::Begin(begin_exprs);
            }
        } 
        panic!("Parse Begin, unexpected eof");
    }

    fn parse_if(&mut self) -> Scheme {
        let _if = self.remove_top();
        let cond = self.parse_expr();
        let b1 = self.parse_expr();
        let b2 = self.parse_expr();
        let _right = self.remove_top();
        return Scheme::If(Box::new(cond), Box::new(b1), Box::new(b2));
    }
    
    fn parse_let(&mut self) -> Scheme {
        let _let = self.remove_top();
        let _binding_left = self.remove_top();
        let mut bindings = HashMap::new();
        while let Some(ref t) = self.top() {
            if t.token.as_str() != ")" {
                let (var, val) = self.parse_binding();
                bindings.insert(var, val);
            } else {
                let _binding_right = self.remove_top();
                let tail = self.parse_expr();
                let _right = self.remove_top();
                return Scheme::Let (bindings, Box::new(tail));
            }
        }
        panic!("Parse let, unexpected eof");
    }

    fn parse_binding(&mut self) -> (String, Scheme) {
        let _left = self.remove_top();
        let var = self.remove_top().unwrap().token;
        let val = self.parse_expr();
        let _right = self.remove_top();
        return (var, val);
    }



    fn parse_funcall(&mut self) -> Scheme {
        let func = self.parse_expr();
        let mut args = vec![];
        while let Some(ref t) = self.top() {
            if t.token.as_str() != ")" {
                let expr = self.parse_expr();
                args.push(expr);
            } else {
                let _right = self.remove_top();
                return Scheme::Funcall (Box::new(func), args);
            }
        } 
        panic!("Parse Funcall, unexpected eof");
    }

    fn parse_set(&mut self) -> Scheme {
        let _set = self.remove_top();
        let e1 = self.parse_expr();
        let e2 = self.parse_expr();
        let _right = self.remove_top();
        Scheme::Set(Box::new(e1), Box::new(e2))
    }

    fn parse_prim1(&mut self) -> Scheme {
        let op = self.remove_top();
        let e1 = self.parse_expr();
        let _right = self.remove_top();
        Scheme::Prim1(op.unwrap().token, Box::new(e1))
    }


    fn parse_prim2(&mut self) -> Scheme {
        let op = self.remove_top();
        let e1 = self.parse_expr();
        let e2 = self.parse_expr();
        let _right = self.remove_top();
        Scheme::Prim2(op.unwrap().token, Box::new(e1), Box::new(e2))
    }

    fn parse_prim3(&mut self) -> Scheme {
        let op = self.remove_top();
        let e1 = self.parse_expr();
        let e2 = self.parse_expr();
        let e3 = self.parse_expr();
        let _right = self.remove_top();
        Scheme::Prim3(op.unwrap().token, Box::new(e1), Box::new(e2), Box::new(e3))
    }

    fn parse_atom(&mut self) -> Scheme {
        let token = &self.top().unwrap().token;
        let chars: Vec<char> = token.chars().collect();
        match chars[0] {
            '\'' => self.parse_quote(),
            e => self.parse_symbol(),
        }
    }

    fn parse_quote(&mut self) -> Scheme {
        let _quote = self.remove_top();
        let t = self.top().unwrap();
        if t.token.as_str() == "(" || t.token.as_str() == "[" {
            return self.parse_quote_list();
        }  
        return self.parse_quote_atom();
    }

    fn parse_quote_atom(&mut self) -> Scheme {
        let atom = self.top().unwrap();
        let chars: Vec<char> = atom.token.chars().collect();
        match chars[0] {
            '#' => self.parse_literal(),
            '0' ..= '9' => Quote (Box::new(self.parse_integer())),
            '-' => Quote (Box::new(self.parse_integer())),
            other => panic!("Invalid literal {} at line {}, col {}", atom.token, atom.line, atom.col),
        }
    }
    
    // right now, we have empty list literal only
    fn parse_quote_list(&mut self) -> Scheme {
        let _left = self.remove_top().unwrap();
        if self.top().unwrap().token.as_str() == ")" { 
            let _right = self.remove_top();
            return Quote (Box::new(EmptyList)); 
        }

        let mut elements = vec![];
        while let Some(ref t) = self.top() {
            match t.token.as_str() {
                ")" => break,
                "(" => elements.push(self.parse_quote_list()),
                "#" => elements.push(self.parse_literal()),
                other => elements.push(self.parse_quote_atom()),
            };
        }
        let _right = self.remove_top();
        let mut list = Quote (Box::new(EmptyList));
        while let Some(scm) = elements.pop() {
            list = Prim2 ("cons".to_string(), Box::new(scm), Box::new(list));
        }
        return list;
    }

    fn parse_symbol(&mut self) -> Scheme {
        let sym = self.remove_top().unwrap();
        if verify_symbol(&sym.token.as_str()) {
            return Scheme::Symbol(sym.token);
        }
        panic!("Invalid Symbol {} at line {} col {}", sym.token, sym.line, sym.col);
    }

    fn parse_integer(&mut self) -> Scheme {
        let num = self.remove_top().unwrap();
        let temp = &num.token.parse();
        match temp {
            Ok(t) => Scheme::Int64(*t),
            Err(e) => panic!("{} not a valid integer", num.token),
        }
    }

    fn parse_literal(&mut self) -> Scheme {
        let _hash = self.remove_top();
        let t = self.top().unwrap();
        match t.token.parse::<usize>() {
            Ok(_len) => self.parse_literal_vector(),
            Err(e) => self.parse_literal_atom(),
        }
    }

    fn parse_literal_atom(&mut self) -> Scheme {
        let atom = self.remove_top().unwrap();
        match atom.token.as_str() {
            "t" => Quote (Box::new(Bool (true))),
            "f" => Quote (Box::new(Bool (false))),
            other => panic!("Invalid literal atom {} at line {}, col {}", other, atom.line, atom.col),
        }
    }

    fn parse_literal_vector(&mut self) -> Scheme {
        let _len = self.remove_top();
        let _left = self.remove_top();
        let mut elements = vec![];
        while let Some(ref t) = self.top() {
            match t.token.as_str() {
                ")" => break,
                "#" => elements.push(self.parse_literal()),
                "(" => elements.push(self.parse_quote_list()),
                other => elements.push(self.parse_quote_atom()),
            };
        }
        let _right = self.remove_top();
        let mut bindings = HashMap::new();
        let tmp = gen_uvar();
        let alloc = prim1_scm("make-vector".to_string(), quote_scm(Int64 (elements.len() as i64)));
        bindings.insert(tmp.clone(), alloc);
        let mut exprs = vec![];
        for (i, v) in elements.into_iter().enumerate() {
            exprs.push(prim3_scm("vector-set!".to_string(), Symbol (tmp.clone()), quote_scm(Int64 (i as i64)), v));
        }
        exprs.push(Symbol (tmp));
        return let_scm(bindings, Begin (exprs));
    }

    fn parse_nop(&mut self) -> Scheme {
        let _nop = self.remove_top();
        let _right = self.remove_top();
        return Scheme::Nop;
    }

    fn parse_bool(&mut self) -> Scheme {
        let s = self.remove_top().unwrap().token;
        let e = match s.as_str() {
            "true" => Scheme::Bool(true),
            "false" => Scheme::Bool(false),
            any => panic!("Invalid bool value {}", any),
        };
        let _right = self.remove_top();
        return e;
    }

    fn parse_void(&mut self) -> Scheme {
        let _nop = self.remove_top();
        let _right = self.remove_top();
        return Scheme::Void;
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
}

