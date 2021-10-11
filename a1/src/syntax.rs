use std::fmt;

#[derive(Debug)]
pub enum Expr {
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


#[derive(Debug)]
pub enum Asm {
    RSP, RBP, RAX, RBX, RCX, RDX, RSI, RDI, 
    R8, R9, R10, R11, R12, R13, R14, R15,
    Imm(i64),
    Op2(String, Box<Asm>, Box<Asm>),
    Retq,
    Cfg(String, Vec<Asm>),
}



impl fmt::Display for Asm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Asm::*;
        match self {
            RAX => write!(f, "%rax"), RBX => write!(f, "%rbx"), RCX => write!(f, "%rcx"), RDX => write!(f, "%rdx"), 
            RSI => write!(f, "%rsi"), RDI => write!(f, "%rdi"), RBP => write!(f, "%rbp"), RSP => write!(f, "%rsp"), 
            R8  => write!(f, "%r8"),  R9  => write!(f, "%r9"),  R10 => write!(f, "%r10"), R11 => write!(f, "%r11"), 
            R12 => write!(f, "%r12"), R13 => write!(f, "%r13"), R14 => write!(f, "%r14"), R15 => write!(f, "%r15"),
            Imm(n) => write!(f, "${}", n),
            Op2(op, box e1, box e2) => write!(f, "\t{} {}, {}\n", op, e1, e2),
            Retq => write!(f, "\tretq\n"),
            Cfg(label, codes) => {
                let mut control_flow_graph = String::from(label);
                control_flow_graph.push_str(":\n");
                for code in codes {
                    let code_str = format!("{}", code);
                    control_flow_graph.push_str(&code_str);
                }
                return write!(f, "{}", control_flow_graph);
            },
            e => write!(f, "DEBUG INFO\n{:?}", e)
        }
    }
}