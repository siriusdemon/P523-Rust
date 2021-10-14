use std::fmt;
use std::collections::HashMap;

#[derive(Debug)]
pub enum Expr {
    Letrec(Vec<Expr>, Box<Expr>),
    Locate(HashMap<String, String>, Box<Expr>),
    Lambda(String, Box<Expr>),
    Begin(Vec<Expr>),
    Prim1(String, Box<Expr>),
    Prim2(String, Box<Expr>, Box<Expr>),
    If(Box<Expr>, Box<Expr>, Box<Expr>),
    If1(Box<Expr>, Box<Expr>),
    Set(Box<Expr>, Box<Expr>),
    Symbol(String),
    Funcall(String),
    Int64(i64),
    Bool(bool),
    Nop,
}


impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Expr::*;
        match self {
            Letrec (lambdas, box body) => {
                let seqs: Vec<String> = lambdas.into_iter().map(|e| format!("{}", e)).collect();
                let seqs_ref: Vec<&str> = seqs.iter().map(|s| s.as_ref()).collect();
                let seqs_s = seqs_ref.join("\n");
                let s = format!("(letrec ({}) \n  {})", seqs_s, body);
                write!(f, "{}", s)
            },
            Lambda (label, box body) => {
                let s = format!("({} (lambda () {}))", label, body);
                write!(f, "{}", s)
            },
            Locate (bindings, box tail) => {
                let seqs: Vec<String> = bindings.iter().map(|(k, v)| format!("({} {})", k, v)).collect();
                let seqs_ref: Vec<&str> = seqs.iter().map(|s| s.as_ref()).collect();
                let seqs_s = seqs_ref.join(" ");
                let s = format!("(locate ({})\n {})", seqs_s, tail);
                write!(f, "{}", s)
            }
            Begin ( exprs ) => {
                let seqs: Vec<String> = exprs.into_iter().map(|e| format!("  {}", e)).collect();
                let seqs_ref: Vec<&str> = seqs.iter().map(|s| s.as_ref()).collect();
                let seqs_s = seqs_ref.join("\n");
                write!(f, "(begin \n{})", seqs_s)
            }
            Set (box e1, box e2) => write!(f, "(set! {} {})", e1, e2),
            Prim1 (op, box e) => write!(f, "({} {})", op, e),
            Prim2 (op, box e1, box e2) => write!(f, "({} {} {})", op, e1, e2),
            If (box cond, box b1, box b2) => write!(f, "(if {} {} {})", cond, b1, b2),
            If1 (box cond, box b) => write!(f, "(if {} {})", cond, b),
            Symbol (s) => write!(f, "{}", s),
            Funcall (name) => write!(f, "({})", name),
            Int64 (i) => write!(f, "{}", i),
            Bool (b) => write!(f, "({})", b),
            Nop => write!(f, "(nop)"),
        }
    }
}


#[derive(Debug)]
pub enum Asm {
    RSP, RBP, RAX, RBX, RCX, RDX, RSI, RDI, 
    R8, R9, R10, R11, R12, R13, R14, R15,
    RIP,
    Imm(i64),
    Label(String),
    Deref(Box<Asm>, i64),
    DerefLabel(Box<Asm>, String),
    Op2(String, Box<Asm>, Box<Asm>),
    Retq,
    Cfg(String, Vec<Asm>),
    Jmp(Box<Asm>),
    Jmpif(String, Box<Asm>),
    Prog(Vec<Asm>),
    Push(Box<Asm>),
    Pop(Box<Asm>),
}


impl fmt::Display for Asm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Asm::*;
        match self {
            RAX => write!(f, "%rax"), RBX => write!(f, "%rbx"), RCX => write!(f, "%rcx"), RDX => write!(f, "%rdx"), 
            RSI => write!(f, "%rsi"), RDI => write!(f, "%rdi"), RBP => write!(f, "%rbp"), RSP => write!(f, "%rsp"), 
            R8  => write!(f, "%r8"),  R9  => write!(f, "%r9"),  R10 => write!(f, "%r10"), R11 => write!(f, "%r11"), 
            R12 => write!(f, "%r12"), R13 => write!(f, "%r13"), R14 => write!(f, "%r14"), R15 => write!(f, "%r15"),
            RIP => write!(f, "%rip"),
            Imm (n) => write!(f, "${}", n),
            Op2 (op, box e1, box e2) => write!(f, "\t{} {}, {}\n", op, e1, e2),
            Deref (box reg, n) => write!(f, "{}({})", n, reg),
            DerefLabel (box reg, s) => write!(f, "{}({})", s, reg),
            Label (s) => write!(f, "{}", s),
            Retq => write!(f, "\tretq\n"),
            Push (box a) => write!(f, "\tpushq {}\n", a),
            Pop (box a) => write!(f, "\tpopq {}\n", a),
            Jmp (box Label(s)) => write!(f, "\tjmp {}\n", s),
            Jmp (box other) => write!(f, "\tjmp *{}\n", other),
            Jmpif (cc, box Label(s)) => write!(f, "\tj{} {}\n", cc, s),
            Jmpif (cc, other) => write!(f, "\tj{} *{}\n", cc, other),
            Cfg (labl, codes) => {
                let mut codes_str = String::new();
                for code in codes {
                    codes_str.push_str( &format!("{}", code) );
                }
                return write!(f, "{}:\n{}", labl, codes_str);
            }
            Prog (cfgs) => {
                let mut codes_str = String::new();
                for cfg in cfgs {
                    codes_str.push_str( &format!("{}\n", cfg) );
                }
                return write!(f, "{}", codes_str);
            }
        }
    }
}