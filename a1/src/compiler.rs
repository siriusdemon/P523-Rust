use std::io::Write;
use std::fs::File;
use crate::syntax::{Expr, Asm};
use crate::parser::{Scanner, Parser};

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

pub struct GenerateX64 {}
impl GenerateX64 {
    pub fn run(&self, expr: Expr, filename: &str) -> std::io::Result<()> {
        let mut file = File::create(filename)?;
        file.write(b".globl _scheme_entry\n")?;
        file.write(b"_scheme_entry:\n")?;
        self.emit_x64(expr, &mut file)?;
        file.write(b"ret\n")?;
        return Ok(());
    }

    fn emit_x64(&self, code: Expr, file: &mut File) -> std::io::Result<()> {
        use Expr::*;
        match code {
            Begin(stats) => {
                for stat in stats {
                    self.emit_set(stat, file);
                }
            },
            _ => panic!("something wrong"),
        };
        Ok(())        
    }

    fn emit_set(&self, code: Expr, file: &mut File) -> std::io::Result<usize> {
        use Expr::*;
        match code {
            Set (box var, box Int64(i)) => {
                let c = format!("\tmovq \t${}, \t%{}\n", i, var);
                file.write(c.as_bytes())
            },
            Set (box v1, box Prim2(op, box v2, box Int64(i))) => {
                let sym = self.x64_binop(&op);
                let c = format!("\t{} \t${}, \t%{}\n", sym, i, v1);
                file.write(c.as_bytes())
            },
            Set (box v1, box Prim2(op, box v2, box v3)) => {
                let sym = self.x64_binop(&op);
                let c = format!("\t{} \t%{}, \t%{}\n", sym, v3, v1);
                file.write(c.as_bytes())
            },
            Set (box v1, box v2) => {
                let c = format!("\tmovq \t%{}, \t%{}\n", v2, v1);
                file.write(c.as_bytes())
            },
            _ => panic!("Expect Set, found {}", code),
        } 
    }

    fn x64_binop(&self, op: &str) -> &str {
        match op {
            "+" => "addq", "-" => "subq", "*" => "imulq",
            _ => panic!("unsupport op {}", op),
        }
    }
}



pub fn compile(s: &str, filename: &str) -> std::io::Result<()>  {
    let expr = ParsePass{}.run(s);
    println!("{}", expr);
    GenerateX64{}.run(expr, filename)
}
