use std::io::Write;
use std::fs::File;
use crate::syntax::{Expr, Asm};
use crate::parser::{Scanner, Parser};

use Expr::*;
use Asm::*;

pub struct ParsePass {}
impl ParsePass {
    pub fn run(&self, expr: &str) -> Expr {
        let scanner = Scanner::new(expr);
        let tokens = scanner.scan();
        let parser = Parser::new(tokens);
        let expr = parser.parse();
        return expr;
    }
}

pub struct CompileToAsmPass {}
impl CompileToAsmPass {
    pub fn run(&self, expr: Expr) -> Asm {
        let label = String::from("_scheme_entry");
        let mut codes = vec![];
        match expr {
            Begin(stats) => {
                for stat in stats {
                    let asm_code = self.expr_to_asm(stat);
                    codes.push(asm_code);
                }
            }
            _ => panic!("Invalid Program {}", expr),
        }
        codes.push(Retq);
        return Cfg(label, codes);
    }

    fn op2(&self, op: &str, src: Asm, dst: Asm) -> Asm {
        Op2(op.to_string(), Box::new(src), Box::new(dst))
    }

    fn asm_binop(&self, op: &str) -> &str {
        match op {
            "+" => "addq", "-" => "subq", "*" => "imulq",
            _ => panic!("unsupport op {}", op),
        }
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
            Set (box Symbol(s), box Int64(i)) => {
                let dst = self.string_to_reg(&s);
                return self.op2("movq", Imm(i), dst);
            },
            Set (box Symbol(v1), box Symbol(v2)) => {
                let dst = self.string_to_reg(&v1);
                let src = self.string_to_reg(&v2);
                return self.op2("movq", src, dst);
            },
            Set (box Symbol(s), box Prim2(op, box _, box Int64(i))) => {
                let dst = self.string_to_reg(&s);
                let binop = self.asm_binop(&op);
                return self.op2(binop, Imm(i), dst);
            },
            Set (box Symbol(v1), box Prim2(op, box _, box Symbol(v3))) => {
                let dst = self.string_to_reg(&v1);
                let src = self.string_to_reg(&v3);
                let binop = self.asm_binop(&op);
                return self.op2(binop, src, dst);
            },
            _ => panic!("Expect Set, found {}", expr),
        }
    }
}


pub struct GenerateAsmPass {}
impl GenerateAsmPass {
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


pub fn compile(s: &str, filename: &str) -> std::io::Result<()>  {
    let expr = ParsePass{}.run(s);
    println!("{}", expr);
    let asms = CompileToAsmPass{}.run(expr);
    GenerateAsmPass{}.run(asms, filename)
}
