use std::io::Write;
use std::fs::File;
use std::collections::HashMap;
use std::collections::HashSet;
use std::vec::IntoIter;
use uuid::Uuid;

use crate::syntax::{Expr, Asm, ConflictGraph, Frame};
use crate::parser::{Scanner, Parser};

use Expr::*;
use Asm::*;

// ---------------------- register/frame --------------------------------
const REGISTERS :[&str; 15] = ["rax", "rbx", "rcx", "rdx", "rsi", "rdi", "rbp",
                                "r8" , "r9" , "r10", "r11", "r12", "r13", "r14", "r15" ];
const PARAMETER_REGISTERS :[&str; 2] = ["r8", "r9"];
const CALLER_SAVED_REGISTERS :[&str; 15] = REGISTERS;
const FRAME_POINTER_REGISTER :&str = "rbp";
const RETURN_VALUE_REGISTER :&str = "rax";
const RETRUN_ADDRESS_REGISTER :&str = "r15";
const ALLOCATION_REGISTER :&str = "rdx";

const FRAME_VARS :[&str; 101] = [
    "fv0", "fv1", "fv2", "fv3", "fv4", "fv5", "fv6", "fv7", "fv8", "fv9", "fv10", 
    "fv11", "fv12", "fv13", "fv14", "fv15", "fv16", "fv17", "fv18", "fv19", "fv20", 
    "fv21", "fv22", "fv23", "fv24", "fv25", "fv26", "fv27", "fv28", "fv29", "fv30", 
    "fv31", "fv32", "fv33", "fv34", "fv35", "fv36", "fv37", "fv38", "fv39", "fv40", 
    "fv41", "fv42", "fv43", "fv44", "fv45", "fv46", "fv47", "fv48", "fv49", "fv50", 
    "fv51", "fv52", "fv53", "fv54", "fv55", "fv56", "fv57", "fv58", "fv59", "fv60",
    "fv61", "fv62", "fv63", "fv64", "fv65", "fv66", "fv67", "fv68", "fv69", "fv70",
    "fv71", "fv72", "fv73", "fv74", "fv75", "fv76", "fv77", "fv78", "fv79", "fv80",
    "fv81", "fv82", "fv83", "fv84", "fv85", "fv86", "fv87", "fv88", "fv89", "fv90",
    "fv91", "fv92", "fv93", "fv94", "fv95", "fv96", "fv97", "fv98", "fv99", "fv100",
];

const ALIGN_SHIFT: usize = 3;
// ---------------------- general utils --------------------------------
fn is_reg(reg: &str) -> bool {
    REGISTERS.contains(&reg)
}

fn is_fv(s: &str) -> bool {
    s.starts_with("fv")
}

fn is_nfv(s: &str) -> bool {
    s.starts_with("nfv")
}

fn is_uvar(sym: &str) -> bool {
    match sym.rfind('.') {
        Some(index) => index > 0 && index < sym.len() - 1,
        None => false,
    }
}

fn is_label(sym: &str) -> bool {
    match sym.rfind('$') {
        Some(index) => index > 0 && index < sym.len() - 1,
        None => false,
    }
}

fn fv_to_index(fv: &str) -> i64 {
    fv[2..].parse().unwrap()
}

fn gensym(prefix: &str) -> String {
    let uid = &Uuid::new_v4().to_string()[..8];
    let mut s = String::from(prefix);
    s.push_str(uid);
    return s;
}

fn gen_label() -> String {
    gensym("tmp$")
}

fn gen_uvar() -> String {
    gensym("t.")
}

fn gen_new_fv() -> String {
    gensym("nfv.")
}

fn get_rp(name: &str) -> String {
    format!("rp.{}", name.replace("$", ""))
}

fn get_rp_nontail(name: &str) -> String {
    let salt = gensym("");
    format!("rpnt_{}_{}", name, salt)
}

fn flatten_begin(expr: Expr) -> Expr {
    fn helper(exprs: Vec<Expr>, collector: &mut Vec<Expr>) {
        for e in exprs {
            if let Begin (vee) = e {
                helper(vee, collector);
            } else {
                collector.push(e);
            }
        }
    }
    if let Begin (mut exprs) = expr {
        if exprs.len() == 1 { 
            return exprs.pop().unwrap();
        }
        let mut new_exprs = vec![];
        helper(exprs, &mut new_exprs);
        return Begin (new_exprs); 
    }
    return expr;
}

fn prim2(op: &str, v1: Expr, v2: Expr) -> Expr {
    Prim2 (op.to_string(), Box::new(v1), Box::new(v2))
}

fn set2(dst: Expr, op: String, opv1: Expr, opv2: Expr) -> Expr {
    let prim = Prim2 (op, Box::new(opv1), Box::new(opv2));
    Set (Box::new(dst), Box::new(prim))
}

fn set1(dst: Expr, src: Expr) -> Expr {
    Set (Box::new(dst), Box::new(src))
}

fn if2(pred: Expr, b1: Expr, b2: Expr) -> Expr {
    If (Box::new(pred), Box::new(b1), Box::new(b2))
}

fn make_alloc(e: Expr) -> Expr {
    set2(Symbol (ALLOCATION_REGISTER.to_string()),
        "+".to_string(), Symbol (ALLOCATION_REGISTER.to_string()), e)
}


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

pub struct RemoveComplexOpera {}
impl RemoveComplexOpera {
     fn run(&self, expr: Expr) -> Expr {
        match expr {
            Letrec (lambdas, box body) => {
                let new_lambdas: Vec<Expr> = lambdas.into_iter()
                                                .map(|e| self.helper(e))
                                                .collect();
                let new_body = self.helper(body);
                return Letrec (new_lambdas, Box::new(new_body));
            }
            _ => panic!("Invalid Program {}", expr),
        }
    } 

    fn helper(&self, expr: Expr) -> Expr {
        match expr {
            Lambda (labl, args, box body) => Lambda (labl, args, Box::new(self.helper(body))),
            Locals (mut uvars, box tail) => {
                let new_tail = self.tail_helper(tail, &mut uvars);
                return Locals (uvars, Box::new(new_tail));
            }
            e => e,
        }
    }

    fn tail_helper(&self, tail: Expr, locals: &mut HashSet<String>) -> Expr {
        match tail {
            Prim2 (op, box v1, box v2) => {
                let mut exprs = vec![];
                let triv1 = self.reduce_value(v1, locals, &mut exprs);
                let triv2 = self.reduce_value(v2, locals, &mut exprs);
                let prim2 = Prim2 (op, Box::new(triv1), Box::new(triv2));
                if exprs.len() == 0 { return prim2; }
                exprs.push(prim2);
                return Begin (exprs);
            }
            Funcall (labl, mut args) => {
                let mut exprs = vec![];
                args = args.into_iter().map(|e| self.reduce_value(e, locals, &mut exprs)).collect();
                let funcall = Funcall (labl, args);
                if exprs.len() == 0 { return funcall; }
                exprs.push(funcall);
                return Begin (exprs);
            }
            If (box pred, box b1, box b2) => {
                let new_b1 = self.tail_helper(b1, locals);
                let new_b2 = self.tail_helper(b2, locals);
                let new_pred = self.pred_helper(pred, locals);
                return if2(new_pred, new_b1, new_b2);
            }
            Begin (mut exprs) => {
                let mut tail = exprs.pop().unwrap();
                tail = self.tail_helper(tail, locals);
                exprs = exprs.into_iter().map(|e| self.effect_helper(e, locals)).collect();
                exprs.push(tail);
                return Begin (exprs);
            }
            Alloc (box e) => {
                let mut exprs = vec![];
                let triv = self.reduce_value(e, locals, &mut exprs);
                let alloc = Alloc (Box::new(triv));
                if exprs.len() == 0 { return alloc; }
                exprs.push(alloc);
                return Begin (exprs);
            }
            Mref (box base, box offset) => {
                let mut exprs = vec![];
                let base_ = self.reduce_value(base, locals, &mut exprs);
                let offset_ = self.reduce_value(offset, locals, &mut exprs);
                let mref = Mref (Box::new(base_), Box::new(offset_));
                if exprs.len() == 0 { return mref; }
                exprs.push(mref);
                return Begin (exprs);
            }
            e => e,
        }         
    }    

    fn pred_helper(&self, pred: Expr, locals: &mut HashSet<String>) -> Expr {
        match pred {
            Prim2 (relop, box v1, box v2) => {
                let mut exprs = vec![];
                let new_v1 = self.reduce_value(v1, locals, &mut exprs);
                let new_v2 = self.reduce_value(v2, locals, &mut exprs);
                let prim2 = Prim2 (relop, Box::new(new_v1), Box::new(new_v2));
                if exprs.len() == 0 { return prim2; }
                exprs.push(prim2);
                return Begin (exprs);
            }
            If (box pred, box br1, box br2) => {
                let new_pred = self.pred_helper(pred, locals);
                let new_br1 = self.pred_helper(br1, locals);
                let new_br2 = self.pred_helper(br2, locals);
                return if2(new_pred, new_br1, new_br2);
            }
            Begin (mut exprs) => {
                let mut pred = exprs.pop().unwrap();
                pred = self.pred_helper(pred, locals);
                exprs = exprs.into_iter().map(|e| self.effect_helper(e, locals)).collect();
                exprs.push(pred);
                return Begin(exprs);
            }
            simple => simple,
        }
    }
    
    fn effect_helper(&self, effect: Expr, locals: &mut HashSet<String>) -> Expr {
        match effect {
            Set (box sym, box Prim2 (op, box v1, box v2)) => {
                let mut exprs = vec![];
                let new_v1 = self.reduce_value(v1, locals, &mut exprs);
                let new_v2 = self.reduce_value(v2, locals, &mut exprs);
                let new_set = set2(sym, op, new_v1, new_v2);
                if exprs.len() == 0 { return new_set; }
                exprs.push(new_set);
                return Begin (exprs);
            }
            Set (box sym, box Funcall (labl, mut args)) => {
                let mut exprs = vec![];
                args = args.into_iter().map(|x| self.reduce_value(x, locals, &mut exprs)).collect();
                let new_set = set1(sym, Funcall (labl, args));
                if exprs.len() == 0 { return new_set; }
                exprs.push(new_set);
                return Begin (exprs);
            }
            Set (box sym, box Alloc (box e)) => {
                let mut exprs = vec![];
                let e = self.reduce_value(e, locals, &mut exprs);
                let new_set = set1(sym, Alloc (Box::new(e)));
                if exprs.len() == 0 { return new_set; }
                exprs.push(new_set);
                return Begin (exprs);
            }
            Set (box sym, box Mref (box base, box offset)) => {
                let mut exprs = vec![];
                let base_ = self.reduce_value(base, locals, &mut exprs);
                let offset_ = self.reduce_value(offset, locals, &mut exprs);
                let new_set = set1(sym, Mref (Box::new(base_), Box::new(offset_)));
                if exprs.len() == 0 { return new_set; }
                exprs.push(new_set);
                return Begin (exprs);
            }
            Mset (box base, box offset, box value) => {
                let mut exprs = vec![];
                let base_ = self.reduce_value(base, locals, &mut exprs);
                let offset_ = self.reduce_value(offset, locals, &mut exprs);
                let value_ = self.reduce_value(value, locals, &mut exprs);
                let mset = Mset (Box::new(base_), Box::new(offset_), Box::new(value_));
                if exprs.len() == 0 { return mset; }
                exprs.push(mset);
                return Begin (exprs);
            }
            Set (box sym, box value) => {
                let mut exprs = vec![];
                let new_value = self.reduce_value(value, locals, &mut exprs);
                let new_set = set1(sym, new_value);
                if exprs.len() == 0 { return new_set; }
                exprs.push(new_set);
                return Begin (exprs);
            }
            If (box pred, box br1, box br2) => {
                let new_br1 = self.effect_helper(br1, locals);
                let new_br2 = self.effect_helper(br2, locals);
                let new_pred = self.pred_helper(pred, locals);
                return if2(new_pred, new_br1, new_br2);
            }
            Begin (mut exprs) => {
                exprs = exprs.into_iter().map(|e| self.effect_helper(e, locals)).collect();
                return Begin (exprs);
            }
            Funcall (labl, mut args) => {
                let mut exprs = vec![];
                args = args.into_iter().map(|e| self.reduce_value(e, locals, &mut exprs)).collect();
                let funcall = Funcall (labl, args);
                if exprs.len() == 0 { return funcall; }
                exprs.push(funcall);
                return Begin (exprs);
            }
            e => e,
        }
    }

    // turn value into a triv, expose any code to prelude
    // any call to this function expect a simple triv
    fn reduce_value(&self, value: Expr, locals: &mut HashSet<String>, prelude: &mut Vec<Expr>) -> Expr {
        match value {
            Prim2 (op, box v1, box v2) => {
                let new_v1 = self.reduce_value(v1, locals, prelude);
                let new_v2 = self.reduce_value(v2, locals, prelude);
                let new_uvar = gen_uvar();
                let assign = set2(Symbol (new_uvar.clone()), op, new_v1, new_v2);
                prelude.push(assign);
                locals.insert(new_uvar.clone());
                return Symbol (new_uvar)
            }
            If (box pred, box b1, box b2) => {
                let new_pred = self.pred_helper(pred, locals);
                let mut exprs1 = vec![];
                let mut new_b1 = self.reduce_value(b1, locals, &mut exprs1);
                if exprs1.len() > 0 { 
                    exprs1.push(new_b1);
                    new_b1 = Begin (exprs1);
                }
                let mut exprs2 = vec![];
                let mut new_b2 = self.reduce_value(b2, locals, &mut exprs2);
                if exprs2.len() > 0 { 
                    exprs2.push(new_b2);
                    new_b2 = Begin (exprs2);
                }
                let new_if = if2(new_pred, new_b1, new_b2);
                let new_uvar = gen_uvar();
                let assign = set1(Symbol (new_uvar.clone()), new_if);
                prelude.push(assign);
                locals.insert(new_uvar.clone());
                return Symbol (new_uvar);
            }
            Begin (mut exprs) => {
                let mut value = exprs.pop().unwrap();
                let mut exprs_ = vec![];
                value = self.reduce_value(value, locals, &mut exprs_);
                if exprs_.len() > 0 {
                    exprs_.push(value);
                    value = Begin (exprs_);
                }
                exprs = exprs.into_iter().map(|e| self.effect_helper(e, locals)).collect();
                exprs.push(value);
                let new_begin = Begin (exprs);
                let new_uvar = gen_uvar();
                let assign = set1(Symbol (new_uvar.clone()), new_begin);
                prelude.push(assign);
                locals.insert(new_uvar.clone());
                return Symbol (new_uvar);
            }
            Funcall (labl, mut args) => {
                args = args.into_iter().map(|e| self.reduce_value(e, locals, prelude)).collect();
                let funcall = Funcall (labl, args);
                let new_uvar = gen_uvar();
                let assign = set1(Symbol (new_uvar.clone()), funcall);
                prelude.push(assign);
                locals.insert(new_uvar.clone());
                return Symbol (new_uvar);
            }
            Alloc (box e) => {
                let new_e = self.reduce_value(e, locals, prelude);
                let new_uvar = gen_uvar();
                let assign = set1(Symbol (new_uvar.clone()), new_e);
                prelude.push(assign);
                locals.insert(new_uvar.clone());
                return Symbol (new_uvar)
            }
            Mref (box base, box offset) => {
                let new_base = self.reduce_value(base, locals, prelude);
                let new_offset = self.reduce_value(offset, locals, prelude);
                let new_uvar = gen_uvar();
                let assign = set1(Symbol (new_uvar.clone()), Mref (Box::new(new_base), Box::new(new_offset)));
                prelude.push(assign);
                locals.insert(new_uvar.clone());
                return Symbol (new_uvar);
            }
            simple => simple,
        }
    }
}

pub struct FlattenSet {}
impl FlattenSet {
    fn run(&self, expr: Expr) -> Expr {
        match expr {
            Letrec (lambdas, box body) => {
                let new_lambdas: Vec<Expr> = lambdas.into_iter()
                                                .map(|e| self.helper(e))
                                                .collect();
                let new_body = self.helper(body);
                return Letrec (new_lambdas, Box::new(new_body));
            }
            _ => panic!("Invalid Program {}", expr),
        }
    } 

    fn helper(&self, expr: Expr) -> Expr {
        match expr {
            Lambda (labl, args, box body) => Lambda (labl, args, Box::new(self.helper(body))),
            Locals (uvars, box tail) => Locals (uvars, Box::new(self.tail_helper(tail))),
            e => e,
        }
    }

    fn tail_helper(&self, tail: Expr) -> Expr {
        match tail {
            If (box pred, box b1, box b2) => {
                let new_b1 = self.tail_helper(b1);
                let new_b2 = self.tail_helper(b2);
                let new_pred = self.pred_helper(pred);
                return If (Box::new(new_pred), Box::new(new_b1), Box::new(new_b2));
            }
            Begin (mut exprs) => {
                let mut tail = exprs.pop().unwrap();
                tail = self.tail_helper(tail);
                let mut new_exprs: Vec<Expr> = exprs.into_iter().map(|e| self.effect_helper(e)).collect();
                new_exprs.push(tail);
                return flatten_begin(Begin (new_exprs));
            }
            e => e,
        }
    }

    fn pred_helper(&self, pred: Expr) -> Expr {
        match pred {
            If (box pred, box b1, box b2) => {
                let new_b1 = self.pred_helper(b1);
                let new_b2 = self.pred_helper(b2);
                let new_pred = self.pred_helper(pred);
                return If (Box::new(new_pred), Box::new(new_b1), Box::new(new_b2));
            } 
            Begin (mut exprs) => {
                let mut tail = exprs.pop().unwrap();
                tail = self.pred_helper(tail);
                let mut new_exprs: Vec<_> = exprs.into_iter().map(|e| self.effect_helper(e)).collect();
                new_exprs.push(tail);
                return flatten_begin(Begin (new_exprs));
            },
            e => e,
        }
    }

    fn effect_helper(&self, effect: Expr) -> Expr {
        match effect {
            If (box pred, box b1, box b2) => {
                let new_pred = self.pred_helper(pred);
                let new_b1 = self.effect_helper(b1);
                let new_b2 = self.effect_helper(b2);
                return If (Box::new(new_pred), Box::new(new_b1), Box::new(new_b2));
            }
            Begin (mut exprs) => {
                exprs = exprs.into_iter().map(|e| self.effect_helper(e)).collect();
                return flatten_begin(Begin (exprs));
            }
            Set (box Symbol (sym), box If (box pred, box b1, box b2)) => {
                let new_b1 = self.simplify_set(sym.clone(), b1);
                let new_b2 = self.simplify_set(sym, b2);
                let new_pred = self.pred_helper(pred);
                return If (Box::new(new_pred), Box::new(new_b1), Box::new(new_b2));
            }
            Set (box Symbol (sym), box Begin (mut exprs)) => {
                let mut tail = exprs.pop().unwrap();
                tail = self.simplify_set(sym, tail);
                exprs = exprs.into_iter().map(|e| self.effect_helper(e)).collect();
                exprs.push(tail);
                return flatten_begin(Begin (exprs));
            }
            e => e,
        }
    }

    fn simplify_set(&self, sym: String, expr: Expr) -> Expr {
        match expr {
            If (box pred, box b1, box b2) => {
                let new_b1 = self.simplify_set(sym.clone(), b1);
                let new_b2 = self.simplify_set(sym.clone(), b2);
                let new_pred = self.pred_helper(pred);
                return If (Box::new(new_pred), Box::new(new_b1), Box::new(new_b2));
            }
            Begin (mut exprs) => {
                let mut tail = exprs.pop().unwrap();
                tail = self.simplify_set(sym, tail);
                exprs.push(tail);
                return flatten_begin(Begin (exprs));
            }
            simple => Set (Box::new(Symbol (sym)), Box::new(simple)),
        }
    }
}


pub struct ImposeCallingConvention {}
impl ImposeCallingConvention {
    fn run(&self, expr: Expr) -> Expr {
        match expr {
            Letrec (lambdas, box body) => {
                let new_lambdas: Vec<Expr> = lambdas.into_iter()
                                                .map(|e| self.lambda_helper(e))
                                                .collect();
                let new_body = self.body_helper(body, vec![], "letrec");
                return Letrec (new_lambdas, Box::new(new_body));
            }
            _ => panic!("Invalid Program {}", expr),
        }
    } 

    fn lambda_helper(&self, expr: Expr) -> Expr {
        if let Lambda (labl, args, box body) = expr {
            let new_body = self.body_helper(body, args, &labl);
            return Lambda (labl, vec![], Box::new(new_body));
        }
        unreachable!()
    }

    fn body_helper(&self, expr: Expr, mut args: Vec<String>, rp: &str) -> Expr {
        if let Locals (mut uvars, box tail) = expr {
            uvars.insert(get_rp(rp));
            for arg in args.iter() {
                uvars.insert(arg.to_string());
            }

            let mut exprs =  vec![];
            exprs.push(set1(Symbol (get_rp(rp)), Symbol (RETRUN_ADDRESS_REGISTER.to_string())));

            // spill the args into two parts, one in registers, another in frame vars
            let mut fv_assign = vec![];
            if args.len() > PARAMETER_REGISTERS.len() {
                let fv_args = args.drain(PARAMETER_REGISTERS.len()..);
                for (i, arg) in fv_args.into_iter().enumerate() {
                    fv_assign.push(set1(Symbol (arg), Symbol (FRAME_VARS[i].to_string())));
                }
            }
            for (arg, reg) in args.into_iter().zip(PARAMETER_REGISTERS) {
                exprs.push(set1(Symbol (arg), Symbol (reg.to_string())));
            }
            // assign frame var later if any
            exprs.append(&mut fv_assign);

            let mut new_frame = Frame::new();
            let new_tail = self.tail_helper(tail, rp, &mut new_frame);
            exprs.push(new_tail);
            for lst in new_frame.iter() {
                for var in lst {
                    uvars.insert(var.to_string());
                }
            }
            return Locals (uvars, Box::new(NewFrames (new_frame, Box::new(flatten_begin(Begin (exprs))))));

        }
        unreachable!()
    }

    fn tail_helper(&self, tail: Expr, rp: &str, new_frame: &mut Frame) -> Expr {
        match tail {
            Funcall (labl, mut args) => {
                let mut exprs = vec![];
                let mut liveset = vec![
                    Symbol (FRAME_POINTER_REGISTER.to_string()),
                    Symbol (RETRUN_ADDRESS_REGISTER.to_string()),
                    Symbol (ALLOCATION_REGISTER.to_string()),
                ];
                if args.len() > PARAMETER_REGISTERS.len() {
                    let fv_args = args.drain(PARAMETER_REGISTERS.len()..);
                    for (i, arg) in fv_args.into_iter().enumerate() {
                        exprs.push(set1(Symbol (FRAME_VARS[i].to_string()), arg));
                        liveset.push(Symbol (FRAME_VARS[i].to_string()));
                    }
                }
                for (arg, reg) in args.into_iter().zip(PARAMETER_REGISTERS) {
                    exprs.push(set1(Symbol (reg.to_string()), arg));
                    liveset.push(Symbol (reg.to_string()));
                }
                exprs.push(set1(Symbol (RETRUN_ADDRESS_REGISTER.to_string()), Symbol (get_rp(rp))));
                let new_call = Funcall (labl, liveset);
                exprs.push(new_call);
                return Begin (exprs);
            }
            If (box mut pred, box b1, box b2) => {
                let new_b1 = self.tail_helper(b1, rp, new_frame);
                let new_b2 = self.tail_helper(b2, rp, new_frame);
                pred = self.pred_helper(pred, new_frame);
                return If (Box::new(pred), Box::new(new_b1), Box::new(new_b2));
            }
            Begin (mut exprs) => {
                let mut tail = exprs.pop().unwrap();
                tail = self.tail_helper(tail, rp, new_frame);
                exprs = exprs.into_iter().map(|e| self.effect_helper(e, new_frame)).collect();
                exprs.push(tail);
                return Begin (exprs);
            }
            Prim2 (op, box v1, box v2) => {
                let expr = set2(Symbol (RETURN_VALUE_REGISTER.to_string()), op, v1, v2);
                let args = vec![
                    Symbol (FRAME_POINTER_REGISTER.to_string()),
                    Symbol (RETURN_VALUE_REGISTER.to_string()),
                    Symbol (ALLOCATION_REGISTER.to_string()),
                ];
                let jump = Funcall (get_rp(rp), args);
                return Begin (vec![expr, jump]);
            }
            Alloc (box e) => {
                let mut exprs = vec![
                    set1(Symbol (RETURN_VALUE_REGISTER.to_string()), 
                         Symbol (ALLOCATION_REGISTER.to_string())),
                    make_alloc(e),
                ];
                let args = vec![
                    Symbol (FRAME_POINTER_REGISTER.to_string()),
                    Symbol (RETURN_VALUE_REGISTER.to_string()),
                    Symbol (ALLOCATION_REGISTER.to_string()),
                ];
                let jump = Funcall (get_rp(rp), args);
                exprs.push(jump);
                return Begin (exprs);
            }
            atom => {
                let expr = set1(Symbol (RETURN_VALUE_REGISTER.to_string()), atom);
                let args = vec![
                    Symbol (FRAME_POINTER_REGISTER.to_string()),
                    Symbol (RETURN_VALUE_REGISTER.to_string()),
                    Symbol (ALLOCATION_REGISTER.to_string()),
                ];
                let jump = Funcall (get_rp(rp), args);
                return Begin (vec![expr, jump]);
            }
        }
    }
    
    fn pred_helper(&self, pred: Expr, new_frame: &mut Frame) -> Expr {
        match pred  {
            If (box pred, box b1, box b2) => {
                let new_pred = self.pred_helper(pred, new_frame);
                let new_b1 = self.pred_helper(b1, new_frame);
                let new_b2 = self.pred_helper(b2, new_frame);
                return if2(new_pred, new_b1, new_b2);
            }
            Begin (mut exprs) => {
                let mut pred = exprs.pop().unwrap();
                pred = self.pred_helper(pred, new_frame);
                exprs = exprs.into_iter().map(|e| self.effect_helper(e, new_frame)).collect();
                exprs.push(pred);
                return Begin (exprs);
            }
            e => e,
        }
    }

    fn effect_helper(&self, effect: Expr, new_frame: &mut Frame) -> Expr {
        match effect {
            If (box pred, box b1, box b2) => {
                let new_pred = self.pred_helper(pred, new_frame);
                let new_b1 = self.effect_helper(b1, new_frame);
                let new_b2 = self.effect_helper(b2, new_frame);
                return if2(new_pred, new_b1, new_b2);
            }
            Begin (mut exprs) => {
                exprs = exprs.into_iter().map(|e| self.effect_helper(e, new_frame)).collect();
                return Begin (exprs);
            }
            Funcall (labl, mut args) => {
                let rp_label = get_rp_nontail(&labl);
                let mut exprs = vec![];
                let mut fv_assign = vec![];
                let mut liveset = vec![
                    Symbol (FRAME_POINTER_REGISTER.to_string()),
                    Symbol (RETRUN_ADDRESS_REGISTER.to_string()),
                ];
                if args.len() > PARAMETER_REGISTERS.len() {
                    let mut fvs = vec![];
                    for a in args.drain(PARAMETER_REGISTERS.len()..) {
                        let new_uvar = gen_new_fv(); 
                        fvs.push(new_uvar.clone());
                        liveset.push(Symbol (new_uvar.clone()));
                        fv_assign.push(set1(Symbol (new_uvar), a));
                    }
                    new_frame.insert(fvs);
                }
                exprs.append(&mut fv_assign);
                for (val, reg) in args.into_iter().zip(PARAMETER_REGISTERS) {
                    liveset.push(Symbol (reg.to_string()));
                    exprs.push(set1(Symbol (reg.to_string()), val));
                }
                exprs.push(set1(Symbol (RETRUN_ADDRESS_REGISTER.to_string()), Symbol (rp_label.clone())));
                let new_call = Funcall (labl, liveset);
                exprs.push(new_call);
                ReturnPoint (rp_label, Box::new(Begin (exprs)))
            }
            Set (box sym, box Funcall (labl, args)) => {
                let mut exprs = vec![];
                exprs.push(self.effect_helper(Funcall (labl, args), new_frame));
                exprs.push(set1(sym, Symbol (RETURN_VALUE_REGISTER.to_string())));
                return Begin (exprs);
            },
            Set (box sym, box Alloc (box e)) => {
                let mut exprs = vec![
                    set1(sym, Symbol (ALLOCATION_REGISTER.to_string())),
                    make_alloc(e),
                ];
                return Begin (exprs);
            },
            e => e,
        }
    } 
}

pub trait UncoverConflict {
    fn type_verify(&self, s: &str) -> bool;
    fn uncover_conflict(&self, conflict_graph: ConflictGraph, tail: Expr) -> Expr;
    fn tail_liveset(&self, tail: &Expr, mut liveset: HashSet<String>, conflict_graph: &mut ConflictGraph, 
                                          call_live: &mut HashSet<String>) -> HashSet<String> {
        match tail {
            Funcall (labl, args) => {
                for a in args {
                    if let Symbol(s) = a { 
                        if self.type_verify(s) {
                            liveset.insert(s.to_string()); 
                        }
                    }
                }
                if self.type_verify(labl) || is_uvar(labl) {
                    liveset.insert(labl.to_string());
                }
                return liveset;
            }
            If (box Bool(true), box b1, _) => self.tail_liveset(b1, liveset, conflict_graph, call_live),
            If (box Bool(false), _, box b2) => self.tail_liveset(b2, liveset, conflict_graph, call_live),
            If (box pred, box b1, box b2) => {
                let true_set = self.tail_liveset(b1, liveset.clone(), conflict_graph, call_live);
                let false_set = self.tail_liveset(b2, liveset, conflict_graph, call_live);
                return self.pred_liveset(pred, true_set, false_set, conflict_graph, call_live);
            }
            Begin (exprs) => {
                let exprs_slice = exprs.as_slice();
                let last = exprs.len() - 1;
                liveset = self.tail_liveset(&exprs_slice[last], liveset, conflict_graph, call_live);
                for i in (0..last).rev() {
                    liveset = self.effect_liveset(&exprs_slice[i], liveset, conflict_graph, call_live); 
                }
                return liveset;
            }
            e => panic!("Invalid Tail {}", tail),
        }   
    }

    fn pred_liveset(&self, pred: &Expr, tliveset: HashSet<String>, fliveset: HashSet<String>, conflict_graph: &mut ConflictGraph,
                                        call_live: &mut HashSet<String>) -> HashSet<String> {
        match pred {
            Bool (true) => tliveset,
            Bool (false) => fliveset,
            If (box pred, box b1, box b2) => {
                let new_tliveset = self.pred_liveset(b1, tliveset.clone(), fliveset.clone(), conflict_graph, call_live);
                let new_fliveset = self.pred_liveset(b2, tliveset, fliveset, conflict_graph, call_live);
                return self.pred_liveset(pred, new_tliveset, new_fliveset, conflict_graph, call_live);
            }
            Begin (exprs) => {
                let exprs_slice = exprs.as_slice();
                let last = exprs.len() - 1;
                let mut liveset = self.pred_liveset(&exprs_slice[last], tliveset, fliveset, conflict_graph, call_live);
                for i in (0..last).rev() {
                    liveset = self.effect_liveset(&exprs_slice[i], liveset, conflict_graph, call_live); 
                }
                return liveset;
            }
            Prim2 (relop, box v1, box v2 ) => {
                let mut liveset: HashSet<_> = self.liveset_union(tliveset, fliveset);
                if let Symbol(s) = v1 { if is_uvar(s) || self.type_verify(s) {
                    liveset.insert(s.to_string());    
                }}
                if let Symbol(s) = v2 { if is_uvar(s) || self.type_verify(s) {
                    liveset.insert(s.to_string());
                }}
                return liveset;
            }
            e => panic!("Invalid Pred Expr {}", e),
        }
    }


    fn effect_liveset(&self, effect: &Expr, mut liveset: HashSet<String>, conflict_graph: &mut ConflictGraph,
                                    call_live: &mut HashSet<String>) -> HashSet<String> {
        match effect {
            Nop => liveset,
            If (box Bool(true), box b1, _) => self.effect_liveset(b1, liveset, conflict_graph, call_live),
            If (box Bool(false), _, box b2) => self.effect_liveset(b2, liveset, conflict_graph, call_live),
            If (box pred, box b1, box b2) => {
                let tliveset = self.effect_liveset(b1, liveset.clone(), conflict_graph, call_live);
                let fliveset = self.effect_liveset(b2, liveset, conflict_graph, call_live);
                return self.pred_liveset(pred, tliveset, fliveset, conflict_graph, call_live);
            }
            Begin (exprs) => {
                for e in exprs.iter().rev() {
                    liveset = self.effect_liveset(e, liveset, conflict_graph, call_live);
                }
                return liveset;
            }
            Set (box Symbol(s), box Prim2 (op, box v2, box v3)) => {
                liveset.remove(s);
                self.record_conflicts(s, "", &liveset, conflict_graph);
                if let Symbol(s) = v2 { if is_uvar(s) || self.type_verify(s) {
                    liveset.insert(s.to_string());
                }}
                if let Symbol(s) = v3 { if is_uvar(s) || self.type_verify(s) {
                    liveset.insert(s.to_string());
                }}
                return liveset;
            }
            Set (box Symbol(s), box Mref (box base, box offset)) => {
                liveset.remove(s);
                self.record_conflicts(s, "", &liveset, conflict_graph);
                if let Symbol(s) = base { if is_uvar(s) || self.type_verify(s) {
                    liveset.insert(s.to_string());
                }}
                if let Symbol(s) = offset { if is_uvar(s) || self.type_verify(s) {
                    liveset.insert(s.to_string());
                }}
                return liveset;
            }
            Set (box Symbol(s1), box Symbol(s2)) => {
                liveset.remove(s1);
                self.record_conflicts(s1, s2, &liveset, conflict_graph);
                if is_uvar(s2) || self.type_verify(s2) {
                    liveset.insert(s2.to_string());
                }
                return liveset;
            }
            Set (box Symbol(s), box v2) => {
                liveset.remove(s);
                self.record_conflicts(s, "", &liveset, conflict_graph);
                return liveset;
            }
            Mset (box base, box offset, box value) => {
                for x in [base, offset, value] {
                    if let Symbol(s) = x { if is_uvar(s) || self.type_verify(s) {
                        liveset.insert(s.to_string());
                    }}
                }
                return liveset;
            }
            ReturnPoint (labl, box tail) => {
                // I will collect here, before any update to liveset
                for live in liveset.iter() { if is_fv(live) || is_uvar(live) {
                    call_live.insert(live.to_string());
                }}
                if let Begin (exprs) = tail {
                    let exprs_slice = exprs.as_slice();
                    let last = exprs_slice.len() - 1;
                    if let Funcall (lab, args) = &exprs_slice[last] {
                        for a in args { if let Symbol (s) = a {
                            liveset.insert(s.to_string());
                        }} 
                    }
                }
                liveset = self.tail_liveset(tail, liveset, conflict_graph, call_live);
                return liveset;
            } 
            e => liveset,
        }
    }

    fn liveset_union(&self,  set1: HashSet<String>, set2: HashSet<String>) -> HashSet<String> {
        set1.union(&set2).into_iter().cloned().collect()
    }

    fn record_conflicts(&self, s: &str, mov: &str, liveset: &HashSet<String>, conflict_graph: &mut ConflictGraph) {
        if !(self.type_verify(s) || is_uvar(s)) { return; }
        // every symbol has an entry.
        for live in liveset.iter() {
            if live != mov {
                if let Some(live_entry) = conflict_graph.get_mut(live) {
                    live_entry.insert(s.to_string());
                }
                if let Some(s_entry) = conflict_graph.get_mut(s) {
                    s_entry.insert(live.to_string());
                }
            }
        }
    }
}


pub struct UncoverFrameConflict {}
impl UncoverFrameConflict {
    fn run(&self, expr: Expr) -> Expr {
        match expr {
            Letrec (lambdas, box body) => {
                let new_lambdas: Vec<Expr> = lambdas.into_iter()
                                                .map(|e| self.helper(e))
                                                .collect();
                let new_body = self.helper(body);
                return Letrec (new_lambdas, Box::new(new_body));
            }
            _ => panic!("Invalid Program {}", expr),
        }
    } 

    fn helper(&self, expr: Expr) -> Expr {
        match expr {
            Lambda (labl, args, box body) => Lambda (labl, args, Box::new(self.helper(body))),
            Locals (uvars, box NewFrames (frames, box tail)) => {
                let mut conflict_graph = ConflictGraph::new();
                for uvar in uvars.iter() {
                    conflict_graph.insert(uvar.to_string(), HashSet::new());
                }
                let new_tail = self.uncover_conflict(conflict_graph, tail);
                return Locals (uvars, Box::new(NewFrames (frames, Box::new(new_tail))));
            }
            e => e,
        }
    }
}

impl UncoverConflict for UncoverFrameConflict {
    fn type_verify(&self, s: &str) -> bool {
        is_fv(s)
    }

    fn uncover_conflict(&self, mut conflict_graph: ConflictGraph, tail: Expr) -> Expr {
        let mut call_live = HashSet::new();
        let mut spills = HashSet::new();
        let _liveset = self.tail_liveset(&tail, HashSet::new(), &mut conflict_graph, &mut call_live);
        // spills are variables that going to be spilled into frame locations
        for var in call_live.iter() { if is_uvar(var) { spills.insert(var.to_string()); } }
        Spills (spills, Box::new(FrameConflict (conflict_graph, Box::new(CallLive (call_live, Box::new(tail))))))
    }
}

pub struct PreAssignFrame {}
impl PreAssignFrame {
    pub fn run(&self, expr: Expr) -> Expr {
        match expr {
            Letrec (lambdas, box body) => {
                let new_lambdas = lambdas.into_iter().map(|e| self.helper(e)).collect();
                let new_body = self.helper(body);
                return Letrec (new_lambdas, Box::new(new_body));
            }
            _ => unreachable!(),
        }
    }
    fn helper(&self, expr: Expr) -> Expr {
        match expr {
            Lambda (label, args, box body) => Lambda (label, args, Box::new(self.helper(body))),
            Locals (mut uvars, box NewFrames (frames, box Spills (spills, box FrameConflict (fc_graph, box CallLive (call_live, box tail))))) => {
                let mut bindings = HashMap::new();
                self.assign_frame(spills, &mut bindings, &fc_graph);
                Locals (uvars, Box::new(
                    NewFrames (frames, Box::new(
                        Locate (bindings, Box::new(
                            FrameConflict (fc_graph, Box::new(
                                CallLive (call_live, Box::new(tail))))))))))
            }
            e => e,
        }
    }

    fn assign_frame(&self, mut spills: HashSet<String>, bindings: &mut HashMap<String, String>, fc_graph: &ConflictGraph) {
        if spills.is_empty() { return; }
        for var in spills.drain() {
            let fv = self.find_compatible(&var, bindings, fc_graph);
            bindings.insert(var, fv);
        }
    }

    fn find_compatible(&self, var: &String, bindings: &mut HashMap<String, String>, fc_graph: &ConflictGraph) -> String {
        let mut uncompat: HashSet<&str> = HashSet::new();
        let conflicts = fc_graph.get(var).unwrap();
        for (v, fv) in bindings {
            if conflicts.contains(v) {
                uncompat.insert(fv);
            }
        }
        for fvi in FRAME_VARS {
            if !uncompat.contains(fvi) && !conflicts.contains(fvi) {
                return fvi.to_string();
            }
        }
        panic!("Aha, frame vars is not enough!");
    }
}


pub struct AssignNewFrame {}
impl AssignNewFrame {
    fn run(&self, expr: Expr) -> Expr {
        match expr {
            Letrec (lambdas, box body) => {
                let new_lambdas :Vec<Expr> = lambdas.into_iter().map(|x| self.helper(x)).collect();
                let new_body = self.helper(body);
                return Letrec (new_lambdas, Box::new(new_body));
            }
            _ => panic!("Invalid Program {}", expr),
        }
    }

    fn helper(&self, expr: Expr) -> Expr {
        match expr {
            Lambda (labl, args, box body) => Lambda (labl, args, Box::new(self.helper(body))),
            Locals (mut uvars, box NewFrames (frames, box Locate (mut bindings, box 
                                    FrameConflict (fc_graph, box CallLive (call_live, box tail))))) => {
                let frame_size = self.decide_frame_size(call_live, &bindings);
                self.assign_new_frame(frame_size, frames, &mut bindings, &mut uvars);
                Locals (uvars, Box::new(Ulocals (HashSet::new(), Box::new(
                    Locate (bindings, Box::new(FrameConflict (fc_graph, Box::new(self.tail_helper(tail, frame_size)))))))))
            }
            _ => panic!("Invalid Program {}", expr),
        }
    }

    fn decide_frame_size(&self, call_live: HashSet<String>, bindings: &HashMap<String, String>) -> usize {
        if call_live.is_empty() { return 0; }
        let mut max_fv_index = 0;
        for mut x in call_live.iter() {
            if !is_fv(x) {
                x = bindings.get(x).unwrap();
            }
            let index = fv_to_index(x) as usize;
            max_fv_index = max_fv_index.max(index);
        }
        return max_fv_index + 1;
    }

    fn assign_new_frame(&self, frame_size: usize, mut frames: Frame, bindings: &mut HashMap<String, String>, uvars: &mut HashSet<String>) {
        for parameters in frames.drain() {
            for (i, p) in parameters.into_iter().enumerate() {
                uvars.remove(&p);
                bindings.insert(p, FRAME_VARS[i + frame_size].to_string());
            }
        } 
    }

    fn tail_helper(&self, expr: Expr, frame_size: usize) -> Expr {
        match expr {
            If (box pred, box b1, box b2) => {
                let new_pred = self.pred_helper(pred, frame_size);
                let new_b1 = self.tail_helper(b1, frame_size);
                let new_b2 = self.tail_helper(b2, frame_size);
                return if2(new_pred, new_b1, new_b2);
            }
            Begin (mut exprs) => {
                let mut tail = exprs.pop().unwrap();
                tail = self.tail_helper(tail, frame_size);
                exprs = exprs.into_iter().map(|e| self.effect_helper(e, frame_size)).collect();
                exprs.push(tail);
                return Begin (exprs);
            }
            e => e,
        }
    }

    fn effect_helper(&self, e: Expr, frame_size: usize) -> Expr {
        match e {
            If (box pred, box b1, box b2) => {
                let new_pred = self.pred_helper(pred, frame_size);
                let new_b1 = self.effect_helper(b1, frame_size);
                let new_b2 = self.effect_helper(b2, frame_size);
                return if2(new_pred, new_b1, new_b2);
            }
            Begin (mut exprs) => {
                exprs = exprs.into_iter().map(|e| self.effect_helper(e, frame_size)).collect();
                return Begin (exprs);
            }
            ReturnPoint (labl, expr) => {
                let nb: i64 = (frame_size << ALIGN_SHIFT) as i64;
                let increament = set2(Symbol (FRAME_POINTER_REGISTER.to_string()), 
                                "+".to_string(), Symbol (FRAME_POINTER_REGISTER.to_string()), Int64 (nb));
                let decreament = set2(Symbol (FRAME_POINTER_REGISTER.to_string()), 
                                "-".to_string(), Symbol (FRAME_POINTER_REGISTER.to_string()), Int64 (nb));
                let exprs = vec![
                    increament, 
                    ReturnPoint (labl, expr),
                    decreament,
                ];
                return Begin (exprs);
            }
            e => e,
        }
    }
    fn pred_helper(&self, e: Expr, frame_size: usize) -> Expr {
        match e {
            If (box pred, box b1, box b2) => {
                let new_pred = self.pred_helper(pred, frame_size);
                let new_b1 = self.pred_helper(b1, frame_size);
                let new_b2 = self.pred_helper(b2, frame_size);
                return if2(new_pred, new_b1, new_b2);
            }
            Begin (mut exprs) => {
                let mut pred = exprs.pop().unwrap(); 
                pred = self.pred_helper(pred, frame_size);
                exprs = exprs.into_iter().map(|e| self.effect_helper(e, frame_size)).collect();
                exprs.push(pred);
                return Begin (exprs);
            }
            e => e,
        }
    }

}

pub struct SelectInstructions {}
impl SelectInstructions {
    pub fn run(&self, expr: Expr) -> Expr {
        if let Letrec (lambdas, box body) = expr {
            let new_lambdas: Vec<Expr> = lambdas.into_iter().map(|e| self.helper(e)).collect();
            let new_body = self.helper(body);
            return Letrec (new_lambdas, Box::new(new_body));
        }
        panic!("Invalid Program {}", expr);
    }

    fn helper(&self, expr: Expr) -> Expr {
        match expr {
            Lambda (label, args, box body) => Lambda (label, args, Box::new(self.helper(body))),
            Locals (uvars, box Ulocals (mut unspills, box Locate (bindings, box FrameConflict (conflict_graph, box tail)))) => {
                let new_tail = self.select_instruction_tail(&mut unspills, tail);
                Locals (uvars, Box::new(Ulocals (unspills, Box::new(Locate (bindings, Box::new(FrameConflict (conflict_graph, Box::new(new_tail))))))))
            }
            Locate (bindings, box tail) => Locate (bindings, Box::new(tail)),
            e => panic!("Invalid program {}", e),
        }
    }

    fn select_instruction_tail(&self, unspills: &mut HashSet<String>, tail: Expr) -> Expr {
        match tail {
            Begin (mut exprs) => {
                let mut tail = exprs.pop().unwrap();
                tail = self.select_instruction_tail(unspills, tail);
                let new_effects: Vec<Expr> = exprs.into_iter().map(|e| self.select_instruction_effect(unspills, e)).collect();
                return flatten_begin(Begin (vec![Begin (new_effects), tail]));
            },
            If (box pred, box b1, box b2) => {
                let new_b1 = self.select_instruction_tail(unspills, b1);
                let new_b2 = self.select_instruction_tail(unspills, b2);
                let new_pred = self.select_instruction_pred(unspills, pred);
                return If (Box::new(new_pred), Box::new(new_b1), Box::new(new_b2));
            }
            funcall => funcall,
        }
    }

    fn select_instruction_pred(&self, unspills: &mut HashSet<String>, pred: Expr) ->  Expr {
        match pred {
            Prim2 (relop, box Symbol (a), box Symbol (b)) => self.relop_fv_rewrite(relop, a, b, unspills),
            Prim2 (relop, box Int64 (i), box Symbol (sym)) => {
                match relop.as_str() {
                    ">"  => prim2("<", Symbol (sym), Int64 (i)),
                    ">=" => prim2("<=", Symbol (sym), Int64 (i)),
                    "<"  => prim2(">", Symbol (sym), Int64 (i)),
                    "<=" => prim2(">=", Symbol (sym), Int64 (i)),
                    "="  => prim2("=", Symbol (sym), Int64 (i)),
                    op   => panic!("Invalid relop {}", op),
                }
            }
            If (box pred, box b1, box b2) => {
                let new_b1 = self.select_instruction_pred(unspills, b1);
                let new_b2 = self.select_instruction_pred(unspills, b2);
                let new_pred = self.select_instruction_pred(unspills, pred);
                return If (Box::new(new_pred), Box::new(new_b1), Box::new(new_b2));
            }
            Begin (mut exprs) => {
                let mut pred = exprs.pop().unwrap();
                pred = self.select_instruction_pred(unspills, pred);
                let new_effects: Vec<Expr> = exprs.into_iter().map(|e| self.select_instruction_effect(unspills, e)).collect();
                return flatten_begin(Begin (vec![Begin (new_effects), pred]));
            }
            e => e
        }
    }

    fn select_instruction_effect(&self, unspills: &mut HashSet<String>, effect: Expr) -> Expr {
        match effect {
            Set (box Symbol (a), box Prim2 (op, box Symbol (b), box Symbol (c))) => {
                if a != b && a != c {
                    return self.rewrite(a, op, Symbol (b), Symbol (c), unspills);
                }
                if a == b {
                    return self.set2_fv_rewrite(a, op, b, c, unspills);
                }
                if a == c && self.is_swapable(&op) {
                    return self.set2_fv_rewrite(a, op, c, b, unspills);
                }
                return self.rewrite(a, op, Symbol (b), Symbol (c), unspills);
            }
            Set (box Symbol (a), box Prim2 (op, box Int64 (i), box Symbol (b))) => {
                if a != b {
                    return self.rewrite(a, op, Int64 (i), Symbol (b), unspills);
                }
                if self.is_swapable(&op) {
                    return set2(Symbol (a), op, Symbol (b), Int64 (i));
                }
                return self.rewrite(a, op, Int64 (i), Symbol (b), unspills);
            }
            Set (box Symbol (a), box Prim2 (op, box Symbol (b), box Int64 (i))) => {
                if a == b {
                    return set2(Symbol (a), op, Symbol (b), Int64 (i));
                }
                return self.rewrite(a, op, Symbol (b), Int64 (i), unspills);
            }
            Set (box Symbol (a), box Prim2 (op, box Int64 (i1), box Int64 (i2))) => {
                return self.set2_int_rewrite(a, op, Int64 (i1), Int64 (i2));
            }
            Set (box Symbol (a), box Symbol (b)) => {
                return self.set1_fv_rewrite(a, b, unspills);
            }
            Set (box Symbol (a), box Mref (box Int64 (base), box Int64 (offset))) => {
                return self.mref_int_rewrite(a, Int64 (base), Int64 (offset));
            }
            Set (box Symbol (a), box Mref (box base, box offset)) => {
                return self.mref_fv_rewrite(a, base, offset, unspills);
            }
            Mset (box Int64 (base), box Int64 (offset), box value) => {
                return self.mset_int_rewrite(Int64 (base), Int64 (offset), value, unspills);
            }
            Mset (box base, box offset, box value) => {
                return self.mset_fv_rewrite(base, offset, value, unspills);
            }
            If (box pred, box b1, box b2) => {
                let new_pred = self.select_instruction_pred(unspills, pred);
                let new_b1 = self.select_instruction_effect(unspills, b1);
                let new_b2 = self.select_instruction_effect(unspills, b2);
                return If (Box::new(new_pred), Box::new(new_b1), Box::new(new_b2));
            }
            Begin (exprs) => {
                let new_exprs: Vec<Expr> = exprs.into_iter().map(|e| self.select_instruction_effect(unspills, e)).collect();
                return flatten_begin(Begin (new_exprs));
            }
            ReturnPoint (labl, box mut tail) => {
                tail = self.select_instruction_tail(unspills, tail);
                return ReturnPoint (labl, Box::new(tail));
            }
            e => e,
        }
    }

    fn is_swapable(&self, op: &str) -> bool {
        match op {
            "+" | "*" | "logor" | "logand" => true,
            "-" | "sra" => false,
            e => panic!("Invalid op {}", e),
        }
    }

    fn relop_fv_rewrite(&self, relop: String, a: String, b: String, unspills: &mut HashSet<String>) -> Expr {
        if is_fv(&a) && is_fv(&b) {
            let new_uvar = gen_uvar();
            unspills.insert(new_uvar.clone());
            let expr1 = set1(Symbol (new_uvar.clone()), Symbol (b));
            let expr2 = Prim2 (relop, Box::new(Symbol (a)), Box::new(Symbol (new_uvar)));
            return Begin (vec![expr1, expr2]);
        }
        return Prim2 (relop, Box::new(Symbol (a)), Box::new(Symbol (b)));
    }

    fn set1_fv_rewrite(&self, a: String, b: String, unspills: &mut HashSet<String>) -> Expr {
        if is_fv(&a) && is_fv(&b) {
            let new_uvar = gen_uvar();
            unspills.insert(new_uvar.clone());
            let expr1 = set1(Symbol (new_uvar.clone()), Symbol (b));
            let expr2 = set1(Symbol (a), Symbol (new_uvar));
            return Begin (vec![expr1, expr2]);
        }
        return set1(Symbol (a), Symbol (b));
    }

    fn set2_fv_rewrite(&self, a: String, op: String, b: String, c: String, unspills: &mut HashSet<String>) -> Expr {
        if is_fv(&a) && is_fv(&c) {
            let new_uvar = gen_uvar();
            unspills.insert(new_uvar.clone());
            let expr1 = set1(Symbol (new_uvar.clone()), Symbol (c));
            let expr2 = set2(Symbol (a), op, Symbol (b), Symbol (new_uvar));
            return Begin (vec![expr1, expr2]);
        } 
        return set2(Symbol (a), op, Symbol (b), Symbol (c));
    }

    fn set2_int_rewrite(&self, a: String, op: String, b: Expr, c: Expr) -> Expr {
        let expr1 = set1(Symbol (a.clone()), b); 
        let expr2 = set2(Symbol (a.clone()), op, Symbol (a), c);
        return Begin (vec![expr1, expr2]);
    }
    
    fn replace_fv(&self, expr: Expr, unspills: &mut HashSet<String>, prelude: &mut Vec<Expr>) -> Expr {
        if let Symbol (s) = expr { 
            if is_fv(&s) {
                let new_uvar = gen_uvar();
                unspills.insert(new_uvar.clone());  
                prelude.push(set1(Symbol (new_uvar.clone()), Symbol (s)));
                return Symbol (new_uvar);
            } else { return Symbol (s); } 
        }
        return expr;
    }
    
    fn mref_int_rewrite(&self, a: String, base: Expr, offset: Expr) -> Expr {
        let exprs = vec![
            set1(Symbol (a.clone()), base),
            set1(Symbol (a.clone()), Mref (Box::new(Symbol (a)), Box::new(offset))),
        ];
        return Begin (exprs);
    }

    fn mref_fv_rewrite(&self, a: String, base: Expr, offset: Expr, unspills: &mut HashSet<String>) -> Expr {
        // so, base and offset should not be fv.
        let mut exprs = vec![];
        let new_base = if let Symbol (b) = base { 
            if is_fv(&b) {
                exprs.push(set1(Symbol (a.clone()), Symbol (b)));
                Symbol (a.clone())
            } else { Symbol (b) }
        } else { base };
        let new_offset = if let Symbol (o) = offset { 
            if is_fv(&o) {
                // if base has no use a, then offset can use a.
                let new_uvar = if exprs.len() > 0 { 
                    let new_uvar = gen_uvar();
                    unspills.insert(new_uvar.clone());  
                    new_uvar
                } else { a.clone() };
                exprs.push(set1(Symbol (new_uvar.clone()), Symbol (o)));
                Symbol (new_uvar)
            } else { Symbol (o) }
        } else { offset };
        let new_mref = set1(Symbol (a), Mref (Box::new(new_base), Box::new(new_offset)));
        if exprs.len() == 0 { return new_mref; }
        exprs.push(new_mref);
        return Begin (exprs);
    }

    fn mset_int_rewrite(&self, base: Expr, offset: Expr, value: Expr, unspills: &mut HashSet<String>) -> Expr {
        let mut exprs = vec![];
        let new_uvar = gen_uvar();
        unspills.insert(new_uvar.clone());  
        exprs.push(set1(Symbol (new_uvar.clone()), base)); 
        let new_value = self.replace_fv(value, unspills, &mut exprs);
        let new_mset = Mset (Box::new(Symbol (new_uvar)), Box::new(offset), Box::new(new_value));
        exprs.push(new_mset); 
        return Begin (exprs);
    }

    fn mset_fv_rewrite(&self, base: Expr, offset: Expr, value: Expr, unspills: &mut HashSet<String>) -> Expr {
        let mut exprs = vec![];
        let new_base = self.replace_fv(base, unspills, &mut exprs);
        let new_offset = self.replace_fv(offset, unspills, &mut exprs);
        let new_value = self.replace_fv(value, unspills, &mut exprs);
        let new_mset = Mset (Box::new(new_base), Box::new(new_offset), Box::new(new_value));
        if exprs.len() == 0 { return new_mset; }
        exprs.push(new_mset);
        return Begin (exprs);
    }

    fn rewrite(&self, a: String, op: String, b: Expr, c: Expr, unspills: &mut HashSet<String>) -> Expr {
        let new_uvar = gen_uvar();
        unspills.insert(new_uvar.clone());
        let expr1 = set1(Symbol (new_uvar.clone()), b);
        let expr2 = set2(Symbol (new_uvar.clone()), op, Symbol (new_uvar.clone()), c);
        let expr3 = set1(Symbol (a), Symbol (new_uvar));
        return Begin (vec![expr1, expr2, expr3]);
    }
}

pub struct UncoverRegisterConflict {}
impl UncoverRegisterConflict {
    fn run(&self, expr: Expr) -> Expr {
        match expr {
            Letrec (lambdas, box body) => {
                let new_lambdas: Vec<Expr> = lambdas.into_iter()
                                                .map(|e| self.helper(e))
                                                .collect();
                let new_body = self.helper(body);
                return Letrec (new_lambdas, Box::new(new_body));
            }
            _ => panic!("Invalid Program {}", expr),
        }
    } 

    fn helper(&self, expr: Expr) -> Expr {
        match expr {
            Lambda (labl, args, box body) => Lambda (labl, args, Box::new(self.helper(body))),
            Locals (uvars, box Ulocals (unspills, box Locate (bindings, box FrameConflict (f_conflict_graph, box tail)))) => {
                let mut r_conflict_graph = ConflictGraph::new();
                for u in uvars.iter() {
                    r_conflict_graph.insert(u.to_string(), HashSet::new());
                }
                for u in unspills.iter() {
                    r_conflict_graph.insert(u.to_string(), HashSet::new());
                }
                let new_tail = self.uncover_conflict(r_conflict_graph, tail);
                Locals (uvars, Box::new(Ulocals (unspills, Box::new(Locate (bindings, Box::new(FrameConflict (f_conflict_graph, Box::new(new_tail))))))))
            }
            e => e,
        }
    }
}

impl UncoverConflict for UncoverRegisterConflict {
    fn type_verify(&self, s: &str) -> bool {
        is_reg(s)
    }

    fn uncover_conflict(&self, mut conflict_graph: ConflictGraph, tail: Expr) -> Expr {
        let mut _callset = HashSet::new();
        let _liveset = self.tail_liveset(&tail, HashSet::new(), &mut conflict_graph, &mut _callset);
        return RegisterConflict (conflict_graph, Box::new(tail));
    }
}



pub struct AssignRegister {}
impl AssignRegister {
    pub fn run(&self, expr: Expr) -> Expr {
        match expr {
            Letrec (lambdas, box body) => {
                let new_lambdas = lambdas.into_iter().map(|e| self.helper(e)).collect();
                let new_body = self.helper(body);
                return Letrec (new_lambdas, Box::new(new_body));
            }
            _ => unreachable!(),
        }
    }
    fn helper(&self, expr: Expr) -> Expr {
        match expr {
            Lambda (label, args, box body) => Lambda (label, args, Box::new(self.helper(body))),
            Locals (mut uvars, box Ulocals (mut unspills, box Locate (bindings, box FrameConflict (fc_graph, box RegisterConflict (mut rc_graph, box tail))))) => {
                let mut assigned = HashMap::new();
                let mut spills = HashSet::new();
                let mut uvars_backup = uvars.clone();
                let unspills_backup = unspills.clone();
                self.assign_registers(&mut uvars, &mut unspills, rc_graph, &mut assigned, &mut spills);
                if spills.is_empty() {
                    return Locate (assigned, Box::new(tail));
                }
                // assign fail, spill the uvars
                spills.iter().for_each(|var| {uvars_backup.remove(var);});
                Locals (uvars_backup, Box::new(Ulocals (unspills_backup, 
                    Box::new(Spills (spills, Box::new(Locate (bindings, 
                        Box::new(FrameConflict (fc_graph, Box::new(tail))))))))))
            }
            e => e,
        }
    }
    fn assign_registers(&self, uvars: &mut HashSet<String>, unspills: &mut HashSet<String>, mut conflict_graph: ConflictGraph, assigned: &mut HashMap<String, String>, spills: &mut HashSet<String>) {
        if conflict_graph.len() == 0 { return; }
        let v = self.proposal_var(uvars, unspills, &conflict_graph);
        let conflicts = conflict_graph.remove(&v).unwrap();
        // update conflict_graph and spillable
        for set in conflict_graph.values_mut() {
            set.remove(&v);
        }
        // assign other variable firstly
        self.assign_registers(uvars, unspills, conflict_graph, assigned, spills);
        // assign the picked variables
        if let Some(reg) = self.find_available(conflicts, assigned) {
            assigned.insert(v, reg);
        } else {
            spills.insert(v);
        }
    }

    fn find_low_degree(&self, conflict_graph: &ConflictGraph, vars: &HashSet<String>) -> (String, usize) {
        let mut var = "";
        let mut degree = usize::MAX;
        for v in vars.iter() {
            let list = conflict_graph.get(v).unwrap();
            if list.len() < degree {
                var = v;
                degree = list.len();
            }
        }
        return (var.to_string(), degree);
    }

    // find the low-degree variable, if exists, return it. Else, spills a uvar.
    fn proposal_var(&self, uvars: &mut HashSet<String>, unspills: &mut HashSet<String>, conflict_graph: &ConflictGraph) -> String {
        let k = REGISTERS.len();

        let (uv, uvdegree) = self.find_low_degree(conflict_graph, unspills);
        if uvdegree < k { 
            unspills.remove(&uv);
            return uv;
        }
 
        let (sv, svdegree) = self.find_low_degree(conflict_graph, uvars);
        if svdegree < k { 
            uvars.remove(&sv);
            return sv
        }
        
        // there is no a low degree variable, try to return a uvar
        if sv != "" {
            uvars.remove(&sv);
            return sv;
        }
        unspills.remove(&uv);
        return uv;
    }

    fn find_available(&self, conflict: HashSet<String>, assigned: &HashMap<String, String>) -> Option<String> {
        let mut unavailable: HashSet<&str> = HashSet::new();
        // record the register that its conflicting variables already in use.
        for (var, reg) in assigned {
            if conflict.contains(var) {
                unavailable.insert(reg);
            }
        }
        for reg in REGISTERS {
            if !unavailable.contains(reg) && !conflict.contains(reg) {
                return Some(reg.to_string()); 
            }
        }
        return None;
    }
}


pub struct AssignFrame {}
impl AssignFrame {
    pub fn run(&self, expr: Expr) -> Expr {
        match expr {
            Letrec (lambdas, box body) => {
                let new_lambdas = lambdas.into_iter().map(|e| self.helper(e)).collect();
                let new_body = self.helper(body);
                return Letrec (new_lambdas, Box::new(new_body));
            }
            _ => unreachable!(),
        }
    }
    fn helper(&self, expr: Expr) -> Expr {
        match expr {
            Lambda (label, args, box body) => Lambda (label, args, Box::new(self.helper(body))),
            Locals (mut uvars, box Ulocals (unspills, box Spills (spills, box Locate (mut bindings, box FrameConflict (fc_graph, box tail))))) => {
                self.assign_frame(spills, &mut bindings, &fc_graph);
                Locals (uvars, Box::new(Ulocals (unspills, Box::new(Locate (bindings, Box::new(FrameConflict (fc_graph, Box::new(tail))))))))
            }
            e => e,
        }
    }

    fn assign_frame(&self, mut spills: HashSet<String>, bindings: &mut HashMap<String, String>, fc_graph: &ConflictGraph) {
        if spills.is_empty() { return; }
        for var in spills.drain() {
            let fv = self.find_compatible(&var, bindings, fc_graph);
            bindings.insert(var, fv);
        }
    }

    fn find_compatible(&self, var: &String, bindings: &mut HashMap<String, String>, fc_graph: &ConflictGraph) -> String {
        let mut uncompat: HashSet<&str> = HashSet::new();
        let conflicts = fc_graph.get(var).unwrap();
        for (v, fv) in bindings {
            if conflicts.contains(v) {
                uncompat.insert(fv);
            }
        }
        for fvi in FRAME_VARS {
            if !uncompat.contains(fvi) && !conflicts.contains(fvi) {
                return fvi.to_string();
            }
        }
        panic!("Aha, frame vars is not enough!");
    }
}

pub struct FinalizeFrameLocations {}
impl FinalizeFrameLocations {
    fn run(&self, expr: Expr) -> Expr {
        match expr {
            Letrec (lambdas, box body) => {
                let new_lambdas = lambdas.into_iter().map(|e| self.helper(e)).collect();
                let new_body = self.helper(body);
                return Letrec (new_lambdas, Box::new(new_body));
            }
            _ => unreachable!(),
        }
    }
    fn helper(&self, expr: Expr) -> Expr {
        match expr {
            Lambda (label, args, box body) => Lambda (label, args, Box::new(self.helper(body))),
            Locals (uvars, box Ulocals (unspills, box Locate (bindings, box FrameConflict (fc_graph, box tail)))) => {
                let new_tail = self.finalize_frame_locations(&bindings, tail);
                Locals (uvars, Box::new(Ulocals (unspills, Box::new(Locate (bindings, Box::new(FrameConflict (fc_graph, Box::new(new_tail))))))))
            }
            e => e,
        }
    }
    fn finalize_frame_locations(&self, bindings: &HashMap<String, String>, expr: Expr) -> Expr {
        match expr {
            If (box pred, box b1, box b2) => {
                let new_pred = self.finalize_frame_locations(bindings, pred);
                let new_b1 = self.finalize_frame_locations(bindings, b1);
                let new_b2 = self.finalize_frame_locations(bindings, b2);
                return If ( Box::new(new_pred), Box::new(new_b1), Box::new(new_b2) );
            }
            Begin (exprs) => {
                let new_exprs: Vec<Expr> = exprs.into_iter().map(|e| self.finalize_frame_locations(bindings, e)).collect();
                return Begin (new_exprs);
            }
            Set (box e1, box e2) => {
                let new_e1 = self.finalize_frame_locations(bindings, e1);
                let new_e2 = self.finalize_frame_locations(bindings, e2);
                if let Symbol (s1) = &new_e1 { if let Symbol (s2) = &new_e2 { if s1 == s2 {
                    return Nop;
                }}}
                return Set (Box::new(new_e1), Box::new(new_e2));
            },
            Prim2 (op, box e1, box e2) => {
                let new_e1 = self.finalize_frame_locations(bindings, e1);
                let new_e2 = self.finalize_frame_locations(bindings, e2);
                return Prim2 (op, Box::new(new_e1), Box::new(new_e2));
            },
            Funcall (name, mut args) => {
                args = args.into_iter().map(|e| self.finalize_frame_locations(bindings, e)).collect();
                match bindings.get(&name) {
                    None => Funcall (name, args),
                    Some (loc) => Funcall (loc.to_string(), args),
                }
            },
            Symbol (s) => {
                match bindings.get(&s) {
                    None => Symbol (s),
                    Some (loc) => Symbol (loc.to_string()),
                }
            },
            ReturnPoint (labl, box mut tail) => {
                tail = self.finalize_frame_locations(bindings, tail);
                return ReturnPoint (labl, Box::new(tail));
            } 
            e => e,
        }
    }
}

pub struct DiscardCallLive {}
impl DiscardCallLive {
    pub fn run(&self, expr: Expr) -> Expr {
        if let Letrec (lambdas, box body) = expr {
            let new_lambdas = lambdas.into_iter().map(|e| self.helper(e)).collect();
            let new_body = self.helper(body);
            return Letrec (new_lambdas, Box::new(new_body));
        }
        unreachable!();
    }

    fn helper(&self, expr: Expr) -> Expr {
        match expr {
            Lambda (label, args, box body) => Lambda (label, args, Box::new(self.helper(body))),
            Locate (bindings,  box tail)  =>  Locate (bindings, Box::new(self.tail_helper(tail))),
            _ => unreachable!(),
        }
    }

    fn tail_helper(&self, tail: Expr) -> Expr {
        match tail {
            Funcall (label, _args) => Funcall (label, vec![]),
            If (box pred, box b1, box b2) => {
                let new_pred = self.pred_helper(pred);
                let new_b1 = self.tail_helper(b1);
                let new_b2 = self.tail_helper(b2);
                return if2(new_pred, new_b1, new_b2);
            }
            Begin (mut exprs) => {
                let new_tail = self.tail_helper(exprs.pop().unwrap());
                exprs = exprs.into_iter().map(|e| self.effect_helper(e)).collect();
                exprs.push(new_tail);  
                return Begin (exprs);
            }
            e => panic!("Invalid tail {}", e),
        }
    }

    fn pred_helper(&self, pred: Expr) -> Expr {
        match pred {
            If (box pred, box b1, box b2) => {
                let new_pred = self.pred_helper(pred);
                let new_b1 = self.pred_helper(b1);
                let new_b2 = self.pred_helper(b2);
                return if2(new_pred, new_b1, new_b2);
            }
            Begin (mut exprs) => {
                let mut pred = self.pred_helper(exprs.pop().unwrap());
                exprs = exprs.into_iter().map(|e| self.effect_helper(e)).collect();
                exprs.push(pred);  
                return Begin (exprs);
            }
            e => e,
        }
    }

    fn effect_helper(&self, e: Expr) -> Expr {
        match e {
            If (box pred, box b1, box b2) => {
                let new_pred = self.pred_helper(pred);
                let new_b1 = self.effect_helper(b1);
                let new_b2 = self.effect_helper(b2);
                return if2(new_pred, new_b1, new_b2);
            }
            Begin (mut exprs) => {
                exprs = exprs.into_iter().map(|e| self.effect_helper(e)).collect();
                return Begin (exprs);
            }
            ReturnPoint (labl, box e) => ReturnPoint (labl, Box::new(self.effect_helper(e))),
            Funcall (labl, _args) => Funcall (labl, vec![]),
            e => e, 
        }
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
            Lambda (label, args, box body) => Lambda (label, args, Box::new(self.remove_locate(body))),
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
                if let Symbol (s1) = &new_e1 { if let Symbol (s2) = &new_e2 { if s1 == s2 {
                    return Nop;
                }}}
                return Set (Box::new(new_e1), Box::new(new_e2));
            },
            Mref (box base, box offset) => {
                let new_base = self.replace_uvar(bindings, base);
                let new_offset = self.replace_uvar(bindings, offset);
                return Mref (Box::new(new_base), Box::new(new_offset));
            }
            Mset (box base, box offset, box value) => {
                let new_base = self.replace_uvar(bindings, base);
                let new_offset = self.replace_uvar(bindings, offset);
                let new_value = self.replace_uvar(bindings, value);
                return Mset (Box::new(new_base), Box::new(new_offset), Box::new(new_value));
            }
            Prim2 (op, box e1, box e2) => {
                let new_e1 = self.replace_uvar(bindings, e1);
                let new_e2 = self.replace_uvar(bindings, e2);
                return Prim2 (op, Box::new(new_e1), Box::new(new_e2));
            },
            Funcall (name, args) => {
                match bindings.get(&name) {
                    None => Funcall (name, args),
                    Some (loc) => Funcall (loc.to_string(), args),
                }
            },
            ReturnPoint (labl, box e) => ReturnPoint (labl, Box::new(self.replace_uvar(bindings, e))),
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

pub struct UpdateFrameLocations {}
impl UpdateFrameLocations {
    pub fn run(&self, expr: Expr) -> Expr {
        match expr {
            Letrec (mut lambdas, box mut tail) => {
                lambdas = lambdas.into_iter().map(|e| self.helper(e)).collect();
                tail = self.helper(tail);
                return Letrec (lambdas, Box::new(tail));
            }
            e => panic!("Invalid Program {}", e),
        }
    }

    fn helper(&self, expr: Expr) -> Expr {
        match expr {
            Lambda (labl, args, box tail) => {
                let (tail, offset) = self.tail_helper(tail, 0);
                return Lambda(labl, args, Box::new(tail));
            }
            tail => self.tail_helper(tail, 0).0,
        } 
    }

    fn tail_helper(&self, tail: Expr, mut offset: i64) -> (Expr, i64) {
        match tail {
            Begin (mut exprs) => {
                let mut tail = exprs.pop().unwrap();
                let mut new_exprs = vec![];
                for mut e in exprs {
                    let res = self.effect_helper(e, offset);
                    offset = res.1;
                    new_exprs.push(res.0);
                }
                let (tail, offset) = self.tail_helper(tail, offset);
                new_exprs.push(tail);
                return (Begin (new_exprs), offset);
            }
            If (box pred, box b1, box b2) => {
                let (new_pred, offset) = self.pred_helper(pred, offset);
                let (new_b1, offset_b1) = self.tail_helper(b1, offset);
                let (new_b2, offset_b2) = self.tail_helper(b2, offset);
                assert_eq!(offset_b1, offset_b2);
                return (if2(new_pred, new_b1, new_b2), offset_b1)
            }
            e => (e, offset),
        }
    }

    fn pred_helper(&self, pred: Expr, mut offset: i64) -> (Expr, i64) {
        match pred {
            Begin (mut exprs) => {
                let mut pred = exprs.pop().unwrap();
                let mut new_exprs = vec![];
                for mut e in exprs {
                    let res  = self.effect_helper(e, offset);
                    offset = res.1;
                    new_exprs.push(res.0);
                }
                let (pred, offset) = self.pred_helper(pred, offset);
                new_exprs.push(pred);
                return (Begin (new_exprs), offset);
            }
            If (box pred, box b1, box b2) => {
                let (new_pred, offset) = self.pred_helper(pred, offset);
                let (new_b1, offset_b1) = self.pred_helper(b1, offset);
                let (new_b2, offset_b2) = self.pred_helper(b2, offset);
                assert_eq!(offset_b1, offset_b2);
                return (if2(new_pred, new_b1, new_b2), offset_b1);
            }
            e => (e, offset),
        }
    }

    fn effect_helper(&self, effect: Expr, mut offset: i64) -> (Expr, i64) {
        match effect {
            Begin (mut exprs) => {
                let mut new_exprs = vec![];
                for mut e in exprs {
                    let res  = self.effect_helper(e, offset);
                    offset = res.1;
                    new_exprs.push(res.0);
                }
                return (Begin (new_exprs), offset);
            }
            If (box pred, box b1, box b2) => {
                let (new_pred, offset) = self.effect_helper(pred, offset);
                let (new_b1, offset_b1) = self.effect_helper(b1, offset);
                let (new_b2, offset_b2) = self.effect_helper(b2, offset);
                assert_eq!(offset_b1, offset_b2);
                return (if2(new_pred, new_b1, new_b2), offset_b1);
            }
            Set (box Symbol (fp), box Prim2 (op, box sym_fp, box Int64 (i))) if fp.as_str() == FRAME_POINTER_REGISTER => {
                match op.as_str() {
                    "+" => (set2(Symbol (fp), op, sym_fp, Int64 (i)), offset + i),
                    "-" => (set2(Symbol (fp), op, sym_fp, Int64 (i)), offset - i),
                    any => panic!("Invalid op on fp {}", any),
                }
            }
            ReturnPoint (labl, box Begin (mut exprs)) => {
                let tail = exprs.pop().unwrap();
                let mut new_exprs = vec![];
                for e in exprs {
                    if let Set (box Symbol (mut s1), box any) = e {
                        if is_fv(&s1) { s1 = self.update_location(&s1, offset); }
                        let rhs = match any { 
                            Symbol (s2) if is_fv(&s2) => Symbol ( self.update_location(&s2, offset) ),
                            other => other,
                        };
                        let new_set = set1 (Symbol (s1), rhs);
                        new_exprs.push(new_set);
                    } 
                }
                let (tail, offset) = self.tail_helper(tail, offset);
                new_exprs.push(tail);
                return (ReturnPoint (labl, Box::new(Begin (new_exprs))), offset);
            }
            e => (e, offset),
        }
    }

    fn update_location(&self, fv: &str, offset: i64) -> String {
        let mut fidx = fv_to_index(&fv); 
        fidx = fidx - (offset  >> ALIGN_SHIFT);
        return format!("fv{}", fidx);
    }
}

pub struct ExposeBasicBlocks {}
impl ExposeBasicBlocks {
    pub fn run(&self, expr: Expr) -> Expr {
        match expr {
            Letrec (lambdas, box tail) => {
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
            Lambda (labl, args, box tail) => {
                let new_tail = self.tail_helper(tail, new_lambdas);
                return Lambda (labl, args, Box::new(new_tail));
            }
            e => panic!("Expect Lambda, get {}", e),
        }
    }

    fn tail_helper(&self, e: Expr, new_lambdas: &mut Vec<Expr>) -> Expr {
        match e {
            Begin (mut exprs) => {
                let mut tail = exprs.pop().unwrap();
                tail = self.tail_helper(tail, new_lambdas);
                while let Some(effect) = exprs.pop() {
                    tail = self.effect_helper(effect, tail, new_lambdas);
                }
                return tail;
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
                let mut pred = exprs.pop().unwrap();
                pred = self.pred_helper(pred, lab1, lab2, new_lambdas);
                while let Some(effect) = exprs.pop() {
                    pred = self.effect_helper(effect, pred, new_lambdas);
                }
                return pred;
            }
            Bool (true) => Funcall (lab1.to_string(), vec![]),
            Bool (false) => Funcall (lab2.to_string(), vec![]),
            If (box pred, box br1, box br2) => {
                let new_lab1 = self.gensym();
                let new_br1 = self.pred_helper(br1, lab1, lab2, new_lambdas);
                self.add_binding(&new_lab1, new_br1, new_lambdas);

                let new_lab2 = self.gensym();
                let new_br2 = self.pred_helper(br2, lab1, lab2, new_lambdas);
                self.add_binding(&new_lab2, new_br2, new_lambdas);
                
                return self.pred_helper(pred, &new_lab1, &new_lab2, new_lambdas);
            }
            relop => If (Box::new(relop), Box::new(Funcall(lab1.to_string(), vec![])), Box::new( Funcall(lab2.to_string(), vec![]))),
        }
    }

    fn effect_helper(&self, effect: Expr, mut tail: Expr, new_lambdas: &mut Vec<Expr>) -> Expr {
        match effect {
            Begin (mut exprs) => {
                while let Some(effect) = exprs.pop() {
                    tail = self.effect_helper(effect, tail, new_lambdas);
                }
                return tail;
            }
            If (box Bool(true), box b1, _) => self.effect_helper(b1, tail, new_lambdas),
            If (box Bool(false),  _, box b2) => self.effect_helper(b2, tail, new_lambdas),
            If (box pred, box b1, box b2) => {
                // the join blocks
                let lab_tail = self.gensym();
                self.add_binding(&lab_tail, tail, new_lambdas);
                // first branch, jump to the join block
                let lab1 = self.gensym();
                let new_b1 = self.effect_helper(b1, Funcall (lab_tail.clone(), vec![]), new_lambdas);
                self.add_binding(&lab1, new_b1, new_lambdas);
                // second branch, jump to the join block too
                let lab2 = self.gensym();
                let new_b2 = self.effect_helper(b2, Funcall (lab_tail, vec![]), new_lambdas);
                self.add_binding(&lab2, new_b2, new_lambdas);
                // since a single expr seq break into several blocks, an effect turn into a tail.
                return self.pred_helper(pred, &lab1, &lab2, new_lambdas);
            }
            ReturnPoint (labl, box en_tail) => {
                self.add_binding(&labl, tail, new_lambdas);
                return self.tail_helper(en_tail, new_lambdas);
            }
            Nop => tail,
            e => Begin (vec![e, tail]),
        }
    }

    fn add_binding(&self, label: &str, tail: Expr, new_lambdas: &mut Vec<Expr>) {
        let lambda = Lambda (label.to_string(), vec![], Box::new(tail));
        new_lambdas.push(lambda);        
    }

    fn gensym(&self) -> String {
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
                while let Some(Lambda(label, args, box tail)) = head {
                    let new_tail = self.reduce(tail, &next);
                    let new_lambda = Lambda (label, args, Box::new(new_tail));
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
        if let Some(Lambda (next_lab, args, _tail)) = next.as_ref() {
            match expr {
                Begin (exprs) => {
                    let new_exprs: Vec<Expr>= exprs.into_iter().map(|e| self.reduce(e, next)).collect();
                    return Begin (new_exprs);
                }
                If (relop, box Funcall (lab1, _), lab2) if &lab1 == next_lab => {
                    let not_relop = Prim1 ("not".to_string(), relop);
                    return If1 (Box::new(not_relop), lab2);
                }
                If (relop, lab1, box Funcall (lab2, _)) if &lab2 == next_lab => {
                    return If1 (relop, lab1);
                }
                Funcall (lab, _) if &lab == next_lab => {
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
            Lambda (label, args, box tail) => Lambda (label, args, Box::new(self.flatten(tail))),
            Begin (exprs) => flatten_begin(Begin (exprs)),
            e => e,
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
                    self.op2("leaq", DerefLabel(Box::new(RIP), Box::new(Label ("_scheme_exit".to_string()))), self.string_to_reg(RETRUN_ADDRESS_REGISTER)),
                ];
                codes.append(&mut self.tail_to_asm(tail));
                let cfg = Cfg(label, codes);
                blocks.push(cfg);
               // other code blocks
                for lambda in lambdas {
                    match lambda {
                        Lambda (labl, args, box lambda_tail) => {
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

    fn fv_to_deref(&self, fv :&str) -> Asm {
        let index :i64 = fv[2..].parse().unwrap();
        let fp = self.string_to_reg(FRAME_POINTER_REGISTER);
        return Deref (Box::new(fp), index << ALIGN_SHIFT);
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
            Set (box Symbol(dst), box Symbol(src)) if is_label(&src) && is_reg(&dst) => {
                let src = DerefLabel (Box::new(RIP), Box::new(Label (src)));                
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
            Funcall (s, _) if is_fv(&s) => {
                let deref = self.fv_to_deref(&s);
                return Jmp (Box::new(deref));
            },
            Funcall (s, _) if is_reg(&s) => {
                let reg = self.string_to_reg(&s);
                return Jmp (Box::new(reg));
            },
            Funcall (s, _) => {
                let label = Label (s);
                return Jmp (Box::new(label));
            }
            If1 (box Prim1(op, box Prim2(relop, box v1, box v2)), box Funcall (s, _)) if op.as_str() == "not" => {
                let v1 = self.expr_to_asm_helper(v1);
                let v2 = self.expr_to_asm_helper(v2);
                let cond = self.op2("cmpq", v2, v1);
                let jmp = Jmpif (self.relop_to_cc(&relop, true).to_string(), Box::new(Label (s)));
                return Code (vec![cond, jmp]);
            }
            If1 (box Prim2(relop, box v1, box v2), box Funcall (s, _)) => {
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
            Symbol (s) if is_reg(&s) => self.string_to_reg(&s),
            Symbol (s) if is_fv(&s) => self.fv_to_deref(&s),
            Symbol (s) if is_label(&s) => Label (s),
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


pub fn compile_formatter<T: std::fmt::Display>(s: &str, expr: &T) {
    println!(">>> {}", s);
    println!("----------------------------");
    println!("{}", expr);
    println!("----------------------------\n");
}


pub fn everybody_home(expr: &Expr) -> bool {
    fn body_home(expr: &Expr) -> bool {
        match expr {
            Locate (bindings, tail) => true,
            _ => false,
        }
    }
    fn lambda_home(expr: &Expr) -> bool {
        match expr {
            Lambda (labl, args, box body) => body_home(body),
            e => panic!("Invalid lambda expression {}", e),
        }
    }
    if let Letrec (lambdas, box body) = expr {
        return lambdas.iter().all(|e| lambda_home(e)) && body_home(body)
    }
    panic!("Invalid Program {}", expr);
}

pub fn compile(s: &str, filename: &str) -> std::io::Result<()>  {
    let expr = ParseExpr{}.run(s);
    compile_formatter("ParseExpr", &expr);
    let expr = RemoveComplexOpera{}.run(expr);
    compile_formatter("RemoveComplexOpera", &expr);
    let expr = FlattenSet{}.run(expr);
    compile_formatter("FlattenSet", &expr);
    let expr = ImposeCallingConvention{}.run(expr);
    compile_formatter("ImposeCallingConvention", &expr);
    let expr = UncoverFrameConflict{}.run(expr);
    compile_formatter("UncoverFrameConflict", &expr);
    let expr = PreAssignFrame{}.run(expr);
    compile_formatter("PreAssignFrame", &expr);
    let mut expr = AssignNewFrame{}.run(expr);
    compile_formatter("AssignNewFrame", &expr);
    // let mut loop_id = 1;
    // loop {
    //     println!("The {}-th iteration", loop_id);
    //     loop_id += 1;

        expr = FinalizeFrameLocations{}.run(expr);
        compile_formatter("FinalizeFrameLocations", &expr);
        expr = SelectInstructions{}.run(expr);
        compile_formatter("SelectInstructions", &expr);
        expr = UncoverRegisterConflict{}.run(expr);
        compile_formatter("UncoverRegisterConflict", &expr);
        expr = AssignRegister{}.run(expr);
        compile_formatter("AssignRegister", &expr);

    //     if everybody_home(&expr) {
    //         break;
    //     }

        expr = AssignFrame{}.run(expr);
        compile_formatter("AssignFrame", &expr);
    // }
    let expr = DiscardCallLive{}.run(expr);
    compile_formatter("DiscardCallLive", &expr);
    let expr = FinalizeLocations{}.run(expr);
    compile_formatter("Finalizelocations", &expr);
    let expr = UpdateFrameLocations{}.run(expr);
    compile_formatter("UpdateFrameLocations", &expr);
    let expr = ExposeBasicBlocks{}.run(expr);
    compile_formatter("ExposeBasicBlocks", &expr);
    let expr = OptimizeJump{}.run(expr);
    compile_formatter("OptimizeJump", &expr);
    let expr = FlattenProgram{}.run(expr);
    compile_formatter("FlattenProgram", &expr);
    // let expr = CompileToAsm{}.run(expr);
    // compile_formatter("CompileToAsm", &expr);
    // return GenerateAsm{}.run(expr, filename)
    Ok(())
}