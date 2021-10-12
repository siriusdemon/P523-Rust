use std::io::Write;
use std::fs::File;
use crate::syntax::{Expr, Asm};
use crate::parser::{Scanner, Parser};

use Expr::*;
use Asm::*;

pub struct ParseExpr {}
impl ParseExpr {
    pub fn run(&self, expr: &str) -> Expr {
        let scanner = Scanner::new(expr);
        let tokens = scanner.scan();
        let parser = Parser::new(tokens);
        let expr = parser.parse();
        return expr;
    }
}

pub struct ExposeFrameVar {}
impl ExposeFrameVar {
    pub fn run(&self, expr: Expr) -> Expr {
        match expr {
            Letrec(lambdas, box tail) => {
                let new_lambda: Vec<Expr> = lambdas.into_iter()
                                                .map(|e| self.replace_fv(e))
                                                .collect();
                let new_tail = self.replace_fv(tail);
                return Letrec(new_lambda, Box::new(new_tail));
            },
            _ => panic!("Invalid Program {}", expr),
        }  
    }

    fn is_fv(&self, s: &str) -> bool {
        s.starts_with("fv")
    }

    fn fv_to_disp(&self, fv :&str) -> Expr {
        let v :Vec<&str> = fv.split("fv").collect();
        let index :i64 = v[1].parse().unwrap();
        return Disp ("rbp".to_string(), index * 8);
    }

    fn replace_fv(&self, expr: Expr) -> Expr {
        match expr {
            Lambda (label, box tail) => Lambda (label, Box::new(self.replace_fv(tail))),
            Begin (exprs) => {
                let new_exprs: Vec<Expr> = exprs.into_iter()
                                                .map(|e| self.replace_fv(e))
                                                .collect();
                return Begin (new_exprs); 
            },
            Set (box Symbol(s), any) if self.is_fv(&s) => {
                let disp = self.fv_to_disp(&s);
                return Set (Box::new(disp), any);
            },
            Set (any, box Symbol(s)) if self.is_fv(&s) => {
                let disp = self.fv_to_disp(&s);
                return Set (any, Box::new(disp));
            },
            Set (any, box Prim2(op, box Symbol(s), any2)) if self.is_fv(&s) => {
                let disp = self.fv_to_disp(&s);
                return Set (any, Box::new( Prim2 (op, Box::new(disp), any2)));
            },
            Set (any, box Prim2(op, any2, box Symbol(s))) if self.is_fv(&s) => {
                let disp = self.fv_to_disp(&s);
                return Set (any, Box::new( Prim2 (op, any2, Box::new(disp))));
            },
            e => e,
        } 
    }
}

// diff from P523, we only handed the nested begin
pub struct FlattenProgram {}
impl FlattenProgram {
    pub fn run(&self, expr: Expr) -> Expr {
        match expr {
            Letrec (lambdas, box tail) => {
                let new_lambda: Vec<Expr> = lambdas.into_iter()
                                                .map(|e| self.flatten(e))
                                                .collect();
                let new_tail = self.flatten(tail);
                return Letrec(new_lambda, Box::new(new_tail));
            },
            _ => panic!("Invalid Program {}", expr),
            
        }  
    }

    fn flatten(&self, expr: Expr) -> Expr {
        match expr {
            Lambda (label, box tail) => Lambda (label, Box::new(self.flatten(tail))),
            Begin (exprs) => {
                let mut new_exprs = vec![];
                self.flatten_begin(exprs, &mut new_exprs);
                return Begin (new_exprs); 
            },
            e => e,
        }
    }
    
    fn flatten_begin(&self, ve: Vec<Expr>, res: &mut Vec<Expr>) {
        for e in ve {
            if let Begin (vee) = e {
                self.flatten_begin(vee, res);
            } else {
                res.push(e);
            }
        }
    }
}

pub struct CompileToAsm {}
impl CompileToAsm {
    pub fn run(&self, expr: Expr) -> Asm {
        let mut blocks = vec![];
        match expr {
            Letrec(lambdas, box tail) => {
                let label = String::from("_scheme_entry");
                let codes = self.tail_to_asm(tail);
                let cfg = Cfg(label, codes);
                blocks.push(cfg);
                for lambda in lambdas {
                    match lambda {
                        Lambda (labl, box lambda_tail) => {
                            let codes: Vec<Asm> = self.tail_to_asm(lambda_tail);
                            let cfg = Cfg(labl, codes);
                            blocks.push(cfg);
                        }
                        e => panic!("Expect Lambda, found {}", e),
                    };
                }
            }
            _ => panic!("Invalid Program {}", expr),
        }
        return Prog (blocks);
    }

    fn tail_to_asm(&self, expr: Expr) -> Vec<Asm> {
        match expr {
            Begin (exprs) => {
                let new_exprs: Vec<Asm> = exprs.into_iter()
                                                .map(|e| self.expr_to_asm(e))
                                                .collect();
                return new_exprs;
            },
            e => vec![self.expr_to_asm(e)],
        }
    }

    fn op2(&self, op: &str, src: Asm, dst: Asm) -> Asm {
        Op2(op.to_string(), Box::new(src), Box::new(dst))
    }

    fn asm_binop(&self, op: &str) -> &str {
        match op {
            "+" => "addq", "-" => "subq", "*" => "imulq",
            "logand" => "",  "logxor" => "", "sra" => "",
            _ => panic!("unsupport op {}", op),
        }
    }

    fn is_reg(&self, reg: &str) -> bool {
        let registers = [
            "rax", "rbx", "rcx", "rbx", "rsi", "rdi", "rbp", "rsp", 
            "r8", "r9", "r10", "r11", "r12", "r13", "r14", "r15",
        ];
        return registers.contains(&reg);
    }

    fn string_to_reg(&self, s: &str) -> Asm {
        match s {
            "rax" => RAX, "rbx" => RBX, "rcx" => RCX, "rdx" => RDX,
            "rsi" => RSI, "rdi" => RDI, "rbp" => RBP, "rsp" => RSP,
            "r8"  => R8,  "r9"  => R9,  "r10" => R10, "r11" => R11,
            "r12" => R12, "r13" => R13, "r14" => R14, "r15" => R15,
            _ => panic!("{} is not a valid register!", s), 
        }
    }
    
    fn expr_to_asm(&self, expr: Expr) -> Asm {
        match expr {
            Set (box dst, box Prim2(op, box _, box src)) => {
                let dst = self.expr_to_asm_helper(dst);
                let src = self.expr_to_asm_helper(src);
                let binop = self.asm_binop(&op);
                return self.op2(binop, src, dst);
            },
            Set (box dst, box src) => {
                let dst = self.expr_to_asm_helper(dst);
                let src = self.expr_to_asm_helper(src);
                return self.op2("movq", src, dst);
            },
            Funcall (s) => Jmp (s),
            _ => panic!("Invaild Expr to Asm, {}", expr),
        }
    }

    fn expr_to_asm_helper(&self, expr: Expr) -> Asm {
        match expr {
            Symbol (s) if self.is_reg(&s) => self.string_to_reg(&s),
            Disp (reg, offset) => Deref (Box::new(self.string_to_reg(&reg)), offset),
            Int64 (i) => Imm (i),
            e => panic!("Expect Atom Expr, found {}", e),
        }
    }
}



pub struct GenerateAsm {}
impl GenerateAsm {
    pub fn run(&self, code: Asm, filename: &str) -> std::io::Result<()> {
        let mut file = File::create(filename)?;
        file.write(b".globl _scheme_entry\n")?;
        self.emit_asm(code, &mut file)?;
        return Ok(());
    }

    pub fn emit_asm(&self, code: Asm, file: &mut File) -> std::io::Result<()> {
        match code {
            Cfg(label, insts) => {
                file.write(label.as_bytes())?;
                file.write(b":\n")?;
                for inst in insts {
                    let text = format!("{}", inst);
                    file.write(text.as_bytes())?;
                }
            },
            _ => panic!("something wrong"),
        };
        Ok(())        
    }
}


pub fn compile_formater<T: std::fmt::Display>(s: &str, expr: &T) {
    println!(">>> {}", s);
    println!("----------------------------");
    println!("{}", expr);
    println!("----------------------------\n");
}

pub fn compile(s: &str, filename: &str) -> std::io::Result<()>  {
    let expr = ParseExpr{}.run(s);
    compile_formater("ParseExpr", &expr);
    let expr = ExposeFrameVar{}.run(expr);
    compile_formater("ExposeFrameVar", &expr);
    let expr = FlattenProgram{}.run(expr);
    compile_formater("FlattenProgram", &expr);
    let expr = CompileToAsm{}.run(expr);
    compile_formater("CompileToAsm", &expr);
    Ok(())
}
