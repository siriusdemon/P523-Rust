use std::io::Write;
use std::fs::File;
use std::collections::HashMap;
use std::vec::IntoIter;

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


pub struct FinalizeLocations {}
impl FinalizeLocations {
    pub fn run(&self, expr: Expr) -> Expr {
        match expr {
            Letrec (lambdas, box body) => {
                let new_lambda: Vec<Expr> = lambdas.into_iter()
                                                .map(|e| self.remove_locate(e))
                                                .collect();
                let tail = self.remove_locate(body);
                return Letrec(new_lambda, Box::new(tail));
            }
            _ => panic!("Invalid Program {}", expr),
        }
    }

    fn remove_locate(&self, expr: Expr) -> Expr {
        match expr {
            Lambda (label, box body) => Lambda (label, Box::new(self.remove_locate(body))),
            Locate (bindings, box tail) => self.replace_uvar(&bindings, tail),
            e => e,
        }
    }

    fn replace_uvar(&self, bindings: &HashMap<String, String>, tail: Expr) -> Expr {
        match tail {
            If (box pred, box b1, box b2) => {
                let new_pred = self.replace_uvar(bindings, pred);
                let new_b1 = self.replace_uvar(bindings, b1);
                let new_b2 = self.replace_uvar(bindings, b2);
                return If ( Box::new(new_pred), Box::new(new_b1), Box::new(new_b2) );
            }
            Begin (exprs) => {
                let new_exprs: Vec<Expr> = exprs.into_iter().map(|e| self.replace_uvar(bindings, e)).collect();
                return Begin (new_exprs);
            }
            Set (box e1, box e2) => {
                let new_e1 = self.replace_uvar(bindings, e1);
                let new_e2 = self.replace_uvar(bindings, e2);
                return Set (Box::new(new_e1), Box::new(new_e2));
            },
            Prim2 (op, box e1, box e2) => {
                let new_e1 = self.replace_uvar(bindings, e1);
                let new_e2 = self.replace_uvar(bindings, e2);
                return Prim2 (op, Box::new(new_e1), Box::new(new_e2));
            },
            Funcall (name) => {
                match bindings.get(&name) {
                    None => Funcall (name),
                    Some (loc) => Funcall (loc.to_string()),
                }
            },
            Symbol (s) => {
                match bindings.get(&s) {
                    None => Symbol (s),
                    Some (loc) => Symbol (loc.to_string()),
                }
            },
            e => e,
        }
    }
}


pub struct ExposeBasicBlocks {}
impl ExposeBasicBlocks {
    pub fn run(&self, expr: Expr) -> Expr {
        match expr {
            Letrec (mut lambdas, box tail) => {
                let mut new_lambdas = vec![];
                let (mut lambdas, new_tail) = self.expose_block(lambdas, tail, &mut new_lambdas);
                // since we process the later first, reverse to keep the original order
                while let Some(new_lambda) = new_lambdas.pop() {
                    lambdas.push(new_lambda)
                }
                return Letrec (lambdas, Box::new(new_tail));
            }
            _ => panic!("Invalid Program {}", expr),
        }
    }

    fn expose_block(&self, lambdas: Vec<Expr>, tail: Expr, new_lambdas: &mut Vec<Expr>) -> (Vec<Expr>, Expr) {
        let lambdas = lambdas.into_iter().map(|e| self.lambda_helper(e, new_lambdas)).collect();
        let tail = self.tail_helper(tail, new_lambdas);        
        return (lambdas, tail);
    }

    fn lambda_helper(&self, e: Expr, new_lambdas: &mut Vec<Expr>) -> Expr {
        match e {
            Lambda (labl, box tail) => {
                let new_tail = self.tail_helper(tail, new_lambdas);
                return Lambda (labl, Box::new(new_tail));
            }
            e => panic!("Expect Lambda, get {}", e),
        }
    }

    fn tail_helper(&self, e: Expr, new_lambdas: &mut Vec<Expr>) -> Expr {
        match e {
            Begin (mut exprs) => {
                let tail = exprs.pop().unwrap();
                let new_tail = self.tail_helper(tail, new_lambdas);
                return self.effects_helper(exprs, new_tail, new_lambdas);
            }
            If (box Bool(true), box b1, _) => self.tail_helper(b1, new_lambdas),
            If (box Bool(false), _, box b2) => self.tail_helper(b2, new_lambdas),
            If (box pred, box b1, box b2) => {
                let lab1 = self.gensym();
                let new_b1 = self.tail_helper(b1, new_lambdas); 
                self.add_binding(&lab1, new_b1, new_lambdas);

                let lab2 = self.gensym();
                let new_b2 = self.tail_helper(b2, new_lambdas);
                self.add_binding(&lab2, new_b2, new_lambdas);

                return self.pred_helper(pred, &lab1, &lab2, new_lambdas);
            }
            e => e,
        }
    }

    fn pred_helper(&self, e: Expr, lab1: &str, lab2: &str, new_lambdas: &mut Vec<Expr>) -> Expr {
        match e {
            Begin (mut exprs) => {
                let pred = exprs.pop().unwrap();
                let new_pred = self.pred_helper(pred, lab1, lab2, new_lambdas);
                return self.effects_helper(exprs, new_pred, new_lambdas);
            }
            Bool (true) => Funcall (lab1.to_string()),
            Bool (false) => Funcall (lab2.to_string()),
            If (box pred, box br1, box br2) => {
                let new_lab1 = self.gensym();
                let new_br1 = self.pred_helper(br1, lab1, lab2, new_lambdas);
                self.add_binding(&new_lab1, new_br1, new_lambdas);

                let new_lab2 = self.gensym();
                let new_br2 = self.pred_helper(br2, lab1, lab2, new_lambdas);
                self.add_binding(&new_lab2, new_br2, new_lambdas);
                
                return self.pred_helper(pred, &new_lab1, &new_lab2, new_lambdas);
            }
            relop => If (Box::new(relop), Box::new(Funcall(lab1.to_string())), Box::new( Funcall(lab2.to_string()))),
        }
    }

    fn effects_helper(&self, mut effects: Vec<Expr>, mut tail: Expr, new_lambdas: &mut Vec<Expr>) -> Expr {
        while let Some(effect) = effects.pop() {
            tail = self.effect_helper(effect, tail, new_lambdas);
        }
        return tail;
    }

    fn effect_helper(&self, effect: Expr, tail: Expr, new_lambdas: &mut Vec<Expr>) -> Expr {
        match effect {
            Begin (mut exprs) => {
                let effect = exprs.pop().unwrap();
                let new_tail = self.effect_helper(effect, tail, new_lambdas); 
                return self.effects_helper(exprs, new_tail, new_lambdas);
            }
            If (box Bool(true), box b1, _) => self.effect_helper(b1, tail, new_lambdas),
            If (box Bool(false),  _, box b2) => self.effect_helper(b2, tail, new_lambdas),
            If (box pred, box b1, box b2) => {
                // the join blocks
                let lab_tail = self.gensym();
                self.add_binding(&lab_tail, tail, new_lambdas);
                // first branch, jump to the join block
                let lab1 = self.gensym();
                let new_b1 = self.effect_helper(b1, Funcall (lab_tail.clone()), new_lambdas);
                self.add_binding(&lab1, new_b1, new_lambdas);
                // second branch, jump to the join block too
                let lab2 = self.gensym();
                let new_b2 = self.effect_helper(b2, Funcall (lab_tail), new_lambdas);
                self.add_binding(&lab2, new_b2, new_lambdas);
                // since a single expr seq break into several blocks, an effect turn into a tail.
                return self.pred_helper(pred, &lab1, &lab2, new_lambdas);
            }
            Nop => tail,
            e => Begin (vec![e, tail]),
        }
    }

    fn add_binding(&self, label: &str, tail: Expr, new_lambdas: &mut Vec<Expr>) {
        let lambda = Lambda (label.to_string(), Box::new(tail));
        new_lambdas.push(lambda);        
    }

    fn gensym(&self) -> String {
        use uuid::Uuid;
        let uid = &Uuid::new_v4().to_string()[..8];
        let mut s = String::from("tmp$");
        s.push_str(uid);
        return s;
    }
}

pub struct OptimizeJump {}
impl OptimizeJump {
    pub fn run(&self, expr: Expr) -> Expr {
        match expr {
            Letrec (lambdas, box tail) => {
                let mut new_lambdas = vec![];
                let mut rest = lambdas.into_iter();
                let mut head = rest.next();
                let mut next = rest.next();
                // main block
                let letrec_tail = self.reduce(tail, &head);
                // lambda block
                while let Some(Lambda(label, box tail)) = head {
                    let new_tail = self.reduce(tail, &next);
                    let new_lambda = Lambda (label, Box::new(new_tail));
                    new_lambdas.push(new_lambda);
                    head = next;
                    next = rest.next();
                }
                return Letrec (new_lambdas, Box::new(letrec_tail));
            }
            e => panic!("Invalid Program {}", e),
        } 
    }

    fn reduce(&self, expr: Expr, next: &Option<Expr>) -> Expr {
        if let Some(Lambda (next_lab, _tail)) = next.as_ref() {
            match expr {
                Begin (exprs) => {
                    let new_exprs: Vec<Expr>= exprs.into_iter().map(|e| self.reduce(e, next)).collect();
                    return Begin (new_exprs);
                }
                If (relop, box Funcall (lab1), lab2) if &lab1 == next_lab => {
                    let not_relop = Prim1 ("not".to_string(), relop);
                    return If1 (Box::new(not_relop), lab2);
                }
                If (relop, lab1, box Funcall (lab2)) if &lab2 == next_lab => {
                    return If1 (relop, lab1);
                }
                Funcall (lab) if &lab == next_lab => {
                    return Nop;
                }
                e => { return self.reduce_if2(e); }
            };
        }
        return self.reduce_if2(expr);
    }
    fn reduce_if2(&self, expr: Expr) -> Expr {
        match expr {
            Begin (exprs) => {
                let new_exprs: Vec<Expr> = exprs.into_iter().map(|e| self.reduce_if2(e)).collect();
                return Begin (new_exprs);
            }
            If (relop, lab1, box lab2) => {
                let if1 = If1 (relop, lab1);
                return Begin (vec![if1, lab2]);
            }
            e => e,
        }
    }
}


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
                // the entry code
                let label = String::from("_scheme_entry");
                let mut codes = vec![
                    Push (Box::new(RBX)),
                    Push (Box::new(RBP)),
                    Push (Box::new(R12)),
                    Push (Box::new(R13)),
                    Push (Box::new(R14)),
                    Push (Box::new(R15)),
                    self.op2("movq", RDI, RBP),
                    self.op2("leaq", DerefLabel(Box::new(RIP), "_scheme_exit".to_string()), R15),
                ];
                codes.append(&mut self.tail_to_asm(tail));
                let cfg = Cfg(label, codes);
                blocks.push(cfg);
               // other code blocks
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
                // the exit code
                let label = String::from("_scheme_exit");
                let codes = vec![
                    Pop (Box::new(R15)),
                    Pop (Box::new(R14)),
                    Pop (Box::new(R13)),
                    Pop (Box::new(R12)),
                    Pop (Box::new(RBP)),
                    Pop (Box::new(RBX)),
                    Retq,
                ];
                let cfg = Cfg(label, codes);
                blocks.push(cfg);
            }
            _ => panic!("Invalid Program {}", expr),
        }
        return Prog (blocks);
    }

    fn tail_to_asm(&self, expr: Expr) -> Vec<Asm> {
        match expr {
            Begin (exprs) => exprs.into_iter().map(|e| self.expr_to_asm(e)).collect(),
            e => vec![self.expr_to_asm(e)],
        }
    }

    fn op2(&self, op: &str, src: Asm, dst: Asm) -> Asm {
        Op2(op.to_string(), Box::new(src), Box::new(dst))
    }

    fn asm_binop(&self, op: &str) -> &str {
        match op {
            "+" => "addq", "-" => "subq", "*" => "imulq",
            "logand" => "andq",  "logor" => "orq", "sra" => "sarq",
            _ => panic!("unsupport op {}", op),
        }
    }

    fn is_fv(&self, s: &str) -> bool {
        s.starts_with("fv")
    }

    fn is_label(&self, sym: &str) -> bool {
        let v: Vec<&str> = sym.split('$').collect();
        v.len() == 2 && v[0].len() > 0 && v[1].len() > 0
    }

    fn fv_to_deref(&self, fv :&str) -> Asm {
        let v :Vec<&str> = fv.split("fv").collect();
        let index :i64 = v[1].parse().unwrap();
        return Deref (Box::new(RBP), index * 8);
    }

    fn is_reg(&self, reg: &str) -> bool {
        let registers = [
            "rax", "rbx", "rcx", "rdx", "rsi", "rdi", "rbp", "rsp", 
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
    
    fn relop_to_cc(&self, s: &str, not: bool) -> &str {
        match s {
            "=" if not => "ne",
            "=" => "e",
            ">" if not => "le",
            ">" => "g",
            "<" if not => "ge",
            "<" => "l",
            "<=" if not => "g",
            "<=" => "le",
            ">=" if not => "l",
            ">=" => "ge",
            op => panic!("Invalid relop {}", op),
        }
    }
    
    fn expr_to_asm(&self, expr: Expr) -> Asm {
        match expr {
            Set (box Symbol(dst), box Symbol(src)) if self.is_label(&src) && self.is_reg(&dst) => {
                let src = DerefLabel (Box::new(RIP), src);                
                let dst = self.string_to_reg(&dst);
                return self.op2("leaq", src, dst);
            },
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
            Funcall (s) if self.is_fv(&s) => {
                let deref = self.fv_to_deref(&s);
                return Jmp (Box::new(deref));
            },
            Funcall (s) if self.is_reg(&s) => {
                let reg = self.string_to_reg(&s);
                return Jmp (Box::new(reg));
            },
            Funcall (s) => {
                let label = Label (s);
                return Jmp (Box::new(label));
            }
            If1 (box Prim1(op, box Prim2(relop, box v1, box v2)), box Funcall (s)) if op.as_str() == "not" => {
                let v1 = self.expr_to_asm_helper(v1);
                let v2 = self.expr_to_asm_helper(v2);
                let cond = self.op2("cmpq", v2, v1);
                let jmp = Jmpif (self.relop_to_cc(&relop, true).to_string(), Box::new(Label (s)));
                return Code (vec![cond, jmp]);
            }
            If1 (box Prim2(relop, box v1, box v2), box Funcall (s)) => {
                let v1 = self.expr_to_asm_helper(v1);
                let v2 = self.expr_to_asm_helper(v2);
                let cond = self.op2("cmpq", v2, v1);
                let jmp = Jmpif (self.relop_to_cc(&relop, false).to_string(), Box::new(Label (s)));
                return Code (vec![cond, jmp]);
            }
            Nop => Code (vec![]),
            _ => panic!("Invaild Expr to Asm, {}", expr),
        }
    }

    fn expr_to_asm_helper(&self, expr: Expr) -> Asm {
        match expr {
            Symbol (s) if self.is_reg(&s) => self.string_to_reg(&s),
            Symbol (s) if self.is_fv(&s) => self.fv_to_deref(&s),
            Symbol (s) => Label (s),
            Int64 (i) => Imm (i),
            e => panic!("Expect Atom Expr, found {}", e),
        }
    }
}



pub struct GenerateAsm {}
impl GenerateAsm {
    pub fn run(&self, code: Asm, filename: &str) -> std::io::Result<()> {
        let mut file = File::create(filename)?;
        let codes = format!("{}", code);
        file.write(b".globl _scheme_entry\n")?;
        file.write(codes.as_bytes())?;
        return Ok(());
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
    // let expr = FinalizeLocations{}.run(expr);
    // compile_formater("FinalizeLocation", &expr);
    // let expr = ExposeBasicBlocks{}.run(expr);
    // compile_formater("ExposeBasicBlocks", &expr);
    // let expr = OptimizeJump{}.run(expr);
    // compile_formater("OptimizeJump", &expr);
    // let expr = FlattenProgram{}.run(expr);
    // compile_formater("FlattenProgram", &expr);
    // let expr = CompileToAsm{}.run(expr);
    // compile_formater("CompileToAsm", &expr);
    // return GenerateAsm{}.run(expr, filename)
    Ok(())
}