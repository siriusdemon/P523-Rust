use std::fmt;
use std::collections::HashMap;
use std::collections::HashSet;

pub type ConflictGraph = HashMap<String, HashSet<String>>;
pub type Frame = HashSet<Vec<String>>;


// ------------------------------- formatter -------------------------------------
fn conflict_graph_formatter(form: &str, conflict_graph: &ConflictGraph, tail: &Expr) -> String {
    let mut cg = vec![];
    for (v, conflicts) in conflict_graph {
        let seqs: Vec<String> = conflicts.iter().map(|c| format!("{}", c)).collect();
        let seqs_ref: Vec<&str> = seqs.iter().map(|s| s.as_ref()).collect();
        let seqs_s = seqs_ref.join(" ");
        let alist = format!("({} {{{}}})", v, seqs_s);
        cg.push(alist);
    }
    let seqs_ref: Vec<&str> = cg.iter().map(|s| s.as_ref()).collect();
    let seqs_s = seqs_ref.join(" ");
    format!("({} ({})\n  {})", form, seqs_s, tail)
}

fn seqs_formatter<E: fmt::Display>(form: &str, seqs: impl Iterator<Item=E>,  join: &str, tail: impl fmt::Display) -> String {
    let seqs: Vec<String> = seqs.map(|e| format!("{}", e)).collect();
    let seqs_ref: Vec<&str> = seqs.iter().map(|s| s.as_ref()).collect();
    let seqs_s = seqs_ref.join(join);
    format!("({} ({})\n  {})", form, seqs_s, tail)
}


// ---------------------- Scheme / Expr / Asm --------------------------------------
#[derive(Debug)]
pub enum Scheme {
    Letrec(Vec<Scheme>, Box<Scheme>),
    Locals(HashSet<String>, Box<Scheme>),
    Let(HashMap<String, Scheme>, Box<Scheme>),
    Lambda(String, Vec<String>, Box<Scheme>),
    Begin(Vec<Scheme>),
    Prim1(String, Box<Scheme>),
    Prim2(String, Box<Scheme>, Box<Scheme>),
    If(Box<Scheme>, Box<Scheme>, Box<Scheme>),
    Set(Box<Scheme>, Box<Scheme>),
    Alloc(Box<Scheme>),
    Mref(Box<Scheme>, Box<Scheme>),
    Mset(Box<Scheme>, Box<Scheme>, Box<Scheme>),
    Symbol(String),
    Funcall(String, Vec<Scheme>),
    Int64(i64),
    Bool(bool),
    Nop,
}


impl fmt::Display for Scheme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Scheme::*;
        match self {
            Letrec (lambdas, box body) => {
                let s = seqs_formatter("letrec", lambdas.iter(), "\n", body);
                write!(f, "{}", s)
            },
            Locals (uvars, box tail) => {
                let s = seqs_formatter("locals", uvars.iter(), " ", tail);
                write!(f, "{}", s)
            }
            Lambda (label, args, box body) => {
                let seqs: Vec<String> = args.iter().map(|e| format!("{}", e)).collect();
                let seqs_ref: Vec<&str> = seqs.iter().map(|s| s.as_ref()).collect();
                let seqs_s = seqs_ref.join(" ");
                let s = format!("({} (lambda ({}) {}))", label, seqs_s, body);
                write!(f, "{}", s)
            },
            Let (bindings, box tail) => {
                 let seqs: Vec<String> = bindings.iter().map(|(k, v)| format!("[{} {}]", k, v)).collect();
                let seqs_ref: Vec<&str> = seqs.iter().map(|s| s.as_ref()).collect();
                let seqs_s = seqs_ref.join(" ");
                let s = format!("(let ({})\n {})", seqs_s, tail);
                write!(f, "{}", s)
            }
            Begin ( exprs ) => {
                let seqs: Vec<String> = exprs.iter().map(|e| format!("  {}", e)).collect();
                let seqs_ref: Vec<&str> = seqs.iter().map(|s| s.as_ref()).collect();
                let seqs_s = seqs_ref.join("\n");
                write!(f, "(begin \n{})", seqs_s)
            }
            Set (box e1, box e2) => write!(f, "(set! {} {})", e1, e2),
            Prim1 (op, box e) => write!(f, "({} {})", op, e),
            Prim2 (op, box e1, box e2) => write!(f, "({} {} {})", op, e1, e2),
            If (box cond, box b1, box b2) => write!(f, "(if {} {} {})", cond, b1, b2),
            Alloc (box e) => write!(f, "(alloc {})", e),
            Mref (box base, box offset) => write!(f, "(mref {} {})", base, offset),
            Mset (box base, box offset, box value) => write!(f, "(mset! {} {} {})", base, offset, value),
            Symbol (s) => write!(f, "{}", s),
            Funcall (name, args) => {
                let seqs: Vec<String> = args.iter().map(|e| format!("{}", e)).collect();
                let seqs_ref: Vec<&str> = seqs.iter().map(|s| s.as_ref()).collect();
                let seqs_s = seqs_ref.join(" ");
                write!(f, "({} {})", name, seqs_s)
            }
            Int64 (i) => write!(f, "{}", i),
            Bool (b) => write!(f, "({})", b),
            Nop => write!(f, "(nop)"),
        }
    }
}



#[derive(Debug)]
pub enum Expr {
    Letrec(Vec<Expr>, Box<Expr>),
    Locals(HashSet<String>, Box<Expr>),
    Ulocals(HashSet<String>, Box<Expr>),
    Spills(HashSet<String>, Box<Expr>),
    Locate(HashMap<String, String>, Box<Expr>),
    Lambda(String, Vec<String>, Box<Expr>),
    RegisterConflict(ConflictGraph, Box<Expr>),
    FrameConflict(ConflictGraph, Box<Expr>),
    NewFrames(Frame, Box<Expr>),
    CallLive(HashSet<String>, Box<Expr>),
    ReturnPoint(String, Box<Expr>),
    Begin(Vec<Expr>),
    Prim1(String, Box<Expr>),
    Prim2(String, Box<Expr>, Box<Expr>),
    If(Box<Expr>, Box<Expr>, Box<Expr>),
    If1(Box<Expr>, Box<Expr>),
    Set(Box<Expr>, Box<Expr>),
    Alloc(Box<Expr>),
    Mref(Box<Expr>, Box<Expr>),
    Mset(Box<Expr>, Box<Expr>, Box<Expr>),
    Symbol(String),
    Funcall(Box<Expr>, Vec<Expr>),
    Int64(i64),
    Bool(bool),
    Nop,
}


impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Expr::*;
        match self {
            Letrec (lambdas, box body) => {
                let s = seqs_formatter("letrec", lambdas.iter(), "\n", body);
                write!(f, "{}", s)
            },
            Locals (uvars, box tail) => {
                let s = seqs_formatter("locals", uvars.iter(), " ", tail);
                write!(f, "{}", s)
            }
            Ulocals (unspills, box tail) => {
                let s = seqs_formatter("ulocals", unspills.iter(), " ", tail);
                write!(f, "{}", s)
            }
            Spills (spills, box tail) => {
                let s = seqs_formatter("spills", spills.iter(), " ", tail);
                write!(f, "{}", s)
            }
            Lambda (label, args, box body) => {
                let seqs: Vec<String> = args.iter().map(|e| format!("{}", e)).collect();
                let seqs_ref: Vec<&str> = seqs.iter().map(|s| s.as_ref()).collect();
                let seqs_s = seqs_ref.join(" ");
                let s = format!("({} (lambda ({}) {}))", label, seqs_s, body);
                write!(f, "{}", s)
            },
            Locate (bindings, box tail) => {
                let seqs: Vec<String> = bindings.iter().map(|(k, v)| format!("({} {})", k, v)).collect();
                let seqs_ref: Vec<&str> = seqs.iter().map(|s| s.as_ref()).collect();
                let seqs_s = seqs_ref.join(" ");
                let s = format!("(locate ({})\n {})", seqs_s, tail);
                write!(f, "{}", s)
            }
            RegisterConflict (conflict_graph, box tail) => {
                let s = conflict_graph_formatter("register-conflict", conflict_graph, tail);
                write!(f, "{}", s)
            }
            FrameConflict (conflict_graph, box tail) => {
                let s = conflict_graph_formatter("frame-conflict", conflict_graph, tail);
                write!(f, "{}", s)
            }
            NewFrames (frames, box tail) => {
                let mut vs = vec![];
                for lst in frames.iter() {
                    let seqs: Vec<String> = lst.iter().map(|e| format!("{}", e)).collect();
                    let seqs_ref: Vec<&str> = seqs.iter().map(|s| s.as_ref()).collect();
                    let seqs_s = seqs_ref.join(" ");
                    vs.push(format!("({})", seqs_s));
                }
                let vs_ref: Vec<&str> = vs.iter().map(|s| s.as_ref()).collect();
                let vs_s = vs_ref.join(" ");
                let s = format!("(new-frames ({}) {})", vs_s, tail);
                write!(f, "{}", s)
            }
            ReturnPoint (rp, box e) => {
                let s = format!("(return-point {} {})", rp, e);
                write!(f, "{}", s)
            }
            CallLive (set, box tail) => {
                let s = seqs_formatter("call-live", set.iter(), " ", tail);
                write!(f, "{}", s)
            }
            Begin ( exprs ) => {
                let seqs: Vec<String> = exprs.iter().map(|e| format!("  {}", e)).collect();
                let seqs_ref: Vec<&str> = seqs.iter().map(|s| s.as_ref()).collect();
                let seqs_s = seqs_ref.join("\n");
                write!(f, "(begin \n{})", seqs_s)
            }
            Set (box e1, box e2) => write!(f, "(set! {} {})", e1, e2),
            Prim1 (op, box e) => write!(f, "({} {})", op, e),
            Prim2 (op, box e1, box e2) => write!(f, "({} {} {})", op, e1, e2),
            If (box cond, box b1, box b2) => write!(f, "(if {} {} {})", cond, b1, b2),
            If1 (box cond, box b) => write!(f, "(if {} {})", cond, b),
            Alloc (box e) => write!(f, "(alloc {})", e),
            Mref (box base, box offset) => write!(f, "(mref {} {})", base, offset),
            Mset (box base, box offset, box value) => write!(f, "(mset! {} {} {})", base, offset, value),
            Symbol (s) => write!(f, "{}", s),
            Funcall (box name, args) => {
                let seqs: Vec<String> = args.iter().map(|e| format!("{}", e)).collect();
                let seqs_ref: Vec<&str> = seqs.iter().map(|s| s.as_ref()).collect();
                let seqs_s = seqs_ref.join(" ");
                write!(f, "({} {})", name, seqs_s)
            }
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
    DerefLabel(Box<Asm>, Box<Asm>),
    DerefRegister(Box<Asm>, Box<Asm>),
    Op2(String, Box<Asm>, Box<Asm>),
    Retq,
    Cfg(String, Vec<Asm>),
    Jmp(Box<Asm>),
    Jmpif(String, Box<Asm>),
    Prog(Vec<Asm>),
    Push(Box<Asm>),
    Pop(Box<Asm>),
    Code(Vec<Asm>),
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
            DerefRegister (box reg1, box reg2) => write!(f, "({},{})", reg1, reg2),
            Label (s) => write!(f, "{}", s.replace("-", "_").replace("?", "q").replace("!", "l")),
            Retq => write!(f, "\tretq\n"),
            Push (box a) => write!(f, "\tpushq {}\n", a),
            Pop (box a) => write!(f, "\tpopq {}\n", a),
            Jmp (box Label(s)) => write!(f, "\tjmp {}\n", s.replace("-", "_").replace("?", "q").replace("!", "l")),
            Jmp (box other) => write!(f, "\tjmp *{}\n", other),
            Jmpif (cc, box Label(s)) => write!(f, "\tj{} {}\n", cc, s),
            Jmpif (cc, other) => write!(f, "\tj{} *{}\n", cc, other),
            Cfg (labl, codes) => {
                let mut codes_str = String::new();
                for code in codes {
                    codes_str.push_str( &format!("{}", code) );
                }
                return write!(f, "{}:\n{}", labl.replace("-", "_").replace("?", "q").replace("!", "l"), codes_str);
            }
            Prog (cfgs) => {
                let mut codes_str = String::new();
                for cfg in cfgs {
                    codes_str.push_str( &format!("{}\n", cfg) );
                }
                return write!(f, "{}", codes_str);
            }
            Code (codes) => {
                let mut codes_str = String::new();
                for code in codes {
                    codes_str.push_str( &format!("{}", code) );
                }
                return write!(f, "{}", codes_str);
            }
        }
    }
}

