use std::fmt;

#[derive(Debug)]
pub enum Expr {
    Int32(i32),
    Int64(i64),
    Begin(Vec<Expr>),
    Prim2(String, Box<Expr>, Box<Expr>),
    Set(Box<Expr>, Box<Expr>),
    Symbol(String),
}


impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Expr::*;
        match self {
            Int32 (i) => write!(f, "{}", i),
            Int64 (i) => write!(f, "{}", i),
            Begin ( exprs ) => {
                let seqs: Vec<String> = exprs.into_iter().map(|e| format!("  {}", e)).collect();
                let seqs_ref: Vec<&str> = seqs.iter().map(|s| s.as_ref()).collect();
                let seqs_s = seqs_ref.join("\n");
                write!(f, "(begin \n{})", seqs_s)
            }
            Set (box e1, box e2) => write!(f, "(set! {} {})", e1, e2),
            Prim2 (op, box e1, box e2) => write!(f, "({} {} {})", op, e1, e2),
            Symbol (s) => write!(f, "{}", s),
        }
    }
}