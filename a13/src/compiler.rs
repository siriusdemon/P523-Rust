use std::io::Write;
use std::fs::File;
use std::collections::HashMap;
use std::collections::HashSet;
use std::vec::IntoIter;

use crate::syntax::{Scheme, Expr, Asm, ConflictGraph, Frame};
use crate::parser::{Scanner, Parser};


use Expr::*;
use Asm::*;

// ---------------------------------------------------------------------
//
// Scheme Language
//
// ---------------------------------------------------------------------

const MASK_FIXNUM  :i64 = 0b111;
const FIXNUM_BITS  :i64 = 61;
const SHIFT_FIXNUM :i64 = 3;
const TAG_FIXNUM   :i64 = 0b000;

const MASK_PAIR  :i64 = 0b111;
const TAG_PAIR   :i64 = 0b001;
const SIZE_PAIR  :i64 = 16;
const CAR_OFFSET :i64 = 0 - TAG_PAIR;
const CDR_OFFSET :i64 = 8 - TAG_PAIR;

const MASK_VECTOR  :i64 = 0b111;
const TAG_VECTOR   :i64 = 0b011;
const VLEN_OFFSET  :i64 = 0 - TAG_VECTOR;
const VDATA_OFFSET :i64 = 8 - TAG_VECTOR;
const DISP_VDATA   :i64 = 8;

const MASK_PROC        :i64 = 0b111;
const TAG_PROC         :i64 = 0b010;
const PROC_CODE_OFFSET :i64 = 0 - TAG_PROC;
const PROC_DATA_OFFSET :i64 = 8 - TAG_PROC;
const DISP_PDATA       :i64 = 8;

const MASK_BOOL :i64 = 0b11110111;
const TAG_BOOL  :i64 = 0b00000110;

const FALSE :i64 = 0b0000_0110;
const TRUE  :i64 = 0b0000_1110;
const NIL   :i64 = 0b0001_0110;
const VOID  :i64 = 0b0001_1110;

fn prim1_scm(op: String, v1: Scheme) -> Scheme {
    Scheme::Prim1 (op, Box::new(v1))
}

fn prim2_scm(op: String, v1: Scheme, v2: Scheme) -> Scheme {
    Scheme::Prim2 (op, Box::new(v1), Box::new(v2))
}

fn prim3_scm(op: String, v1: Scheme, v2: Scheme, v3: Scheme) -> Scheme {
    Scheme::Prim3 (op, Box::new(v1), Box::new(v2), Box::new(v3))
}

fn if2_scm(pred: Scheme, b1: Scheme, b2: Scheme) -> Scheme {
    Scheme::If (Box::new(pred), Box::new(b1), Box::new(b2))
}

fn set1_scm(sym: Scheme, val: Scheme) -> Scheme {
    Scheme::Set (Box::new(sym), Box::new(val))
}

fn mset_scm(v1: Scheme, v2: Scheme, v3: Scheme) -> Scheme {
    Scheme::Mset (Box::new(v1), Box::new(v2), Box::new(v3))
}

fn mref_scm(v1: Scheme, v2: Scheme) -> Scheme {
    Scheme::Mref (Box::new(v1), Box::new(v2))
}

fn let_scm(bindings: HashMap<String, Scheme>, e: Scheme) -> Scheme {
    Scheme::Let (bindings, Box::new(e))
}

fn letrec_scm(bindings: HashMap<String, Scheme>, e: Scheme) -> Scheme {
    Scheme::Letrec (bindings, Box::new(e))
}

fn lambda_scm(args: Vec<String>, e: Scheme) -> Scheme {
    Scheme::Lambda (args, Box::new(e))
}

fn funcall_scm(func: Scheme, args: Vec<Scheme>) -> Scheme {
    Scheme::Funcall (Box::new(func), args)
}

fn quote_scm(scm: Scheme) -> Scheme {
    Scheme::Quote (Box::new(scm))
}

fn uvar_to_label(var: &str) -> String {
    var.replace(".", "$")
}

fn is_value_prim(s: &str) -> bool {
    ["+", "-", "*", "car", "cdr", "cons", "make-vector", "vector-length", "vector-ref", "void", 
    "make-procedure", "procedure-code", "procedure-ref"].contains(&s)
}

fn is_pred_prim(s: &str) -> bool {
    ["<=", "<", "=", ">=", ">", "boolean?", "eq?", "fixnum?", "null?", "pair?", "vector?", "procedure?"].contains(&s)
}

fn is_effect_prim(s: &str) -> bool {
    ["set-car!", "set-cdr!", "vector-set!", "procedure-set!"].contains(&s)
}

fn gen_anon() -> String {
    gensym("anon.")
}

fn make_nopless_begin(exprs: Vec<Scheme>) -> Scheme {
    use Scheme::*;
    fn helper(exprs: Vec<Scheme>, collector: &mut Vec<Scheme>) {
        for e in exprs {
            if let Begin (vee) = e {
                helper(vee, collector);
            } else if let Nop = e {
                // skip nop
            } else {
                collector.push(e);
            }
        }
    }
    let mut new_exprs = vec![];
    helper(exprs, &mut new_exprs);
    if new_exprs.len() == 0 { return Scheme::Nop; }
    Scheme::Begin (new_exprs)
}

pub struct ParseScheme {}
impl ParseScheme {
    pub fn run(&self, scm: &str) -> Scheme {
        let scanner = Scanner::new(scm);
        let tokens = scanner.scan();
        let parser = Parser::new(tokens);
        let scm = parser.parse();
        return scm;
    }
}


pub struct OptimizeDirectCall {}
impl OptimizeDirectCall {
    pub fn run(&self, scm: Scheme) -> Scheme {
        self.optimize(scm)
    }

    fn optimize(&self, scm: Scheme) -> Scheme {
        use Scheme::*;
        match scm {
            If (box pred, box b1, box b2) => if2_scm(
                self.optimize(pred),
                self.optimize(b1),
                self.optimize(b2),
            ),
            Begin (mut exprs) => Begin (
                exprs.into_iter().map(|e| self.optimize(e)).collect()
            ),
            Funcall (box Lambda (args, box body), values) if args.len() == values.len() => {
                let mut bindings = HashMap::new();
                for (arg, val) in args.into_iter().zip(values) {
                    bindings.insert(arg, self.optimize(val));
                }
                return let_scm(bindings, self.optimize(body));
            }
            Funcall (box func, mut values) => funcall_scm(
                self.optimize(func), 
                values.into_iter().map(|e| self.optimize(e)).collect()
            ),
            Let (mut bindings, box body) => {
                let mut new_bindings = HashMap::new();
                for (k, v) in bindings.drain() {
                    new_bindings.insert(k, self.optimize(v));
                }
                return let_scm(new_bindings, self.optimize(body));
            }
            Letrec (mut bindings, box body) => {
                let mut new_bindings = HashMap::new();
                for (k, v) in bindings.drain() {
                    new_bindings.insert(k, self.optimize(v));
                }
                return letrec_scm(new_bindings, self.optimize(body));
            }
            Lambda (args, box body) => lambda_scm(args, self.optimize(body)),
            Prim1 (op, box e) => prim1_scm(op, self.optimize(e)),
            Prim2 (op, box e1, box e2) => prim2_scm(op, self.optimize(e1), self.optimize(e2)),
            Prim3 (op, box e1, box e2, box e3) => prim3_scm(op, self.optimize(e1), self.optimize(e2), self.optimize(e3)),
            Symbol (s) => Symbol (s),
            Quote (box imm) => quote_scm(imm),
            Void => Void,
            other => panic!("Invalid Program {}", other),
        }
    }
}

pub struct RemoveAnonymousLambda {}
impl RemoveAnonymousLambda {
    pub fn run(&self, scm: Scheme) -> Scheme {
        self.remove(scm, true)
    }

    fn remove(&self, scm: Scheme, anonymous: bool) -> Scheme {
        use Scheme::*;
        match scm {
            If (box pred, box b1, box b2) => if2_scm(
                self.remove(pred, true),
                self.remove(b1, true),
                self.remove(b2, true),
            ),
            Begin (mut exprs) => Begin (
                exprs.into_iter().map(|e| self.remove(e, true)).collect()
            ),
            Funcall (box func, mut values) => funcall_scm(
                self.remove(func, true), 
                values.into_iter().map(|e| self.remove(e, true)).collect()
            ),
            Let (mut bindings, box body) => {
                let mut new_bindings = HashMap::new();
                for (k, v) in bindings.drain() {
                    new_bindings.insert(k, self.remove(v, false));
                }
                return let_scm(new_bindings, self.remove(body, true));
            }
            Letrec (mut bindings, box body) => {
                let mut new_bindings = HashMap::new();
                for (k, v) in bindings.drain() {
                    new_bindings.insert(k, self.remove(v, false));
                }
                return letrec_scm(new_bindings, self.remove(body, true));
            }
            Lambda (args, box body) => {
                let scm = lambda_scm(args, self.remove(body, true));
                if anonymous {
                    let tmp = gen_anon();
                    let mut new_bindings = HashMap::new();
                    new_bindings.insert(tmp.clone(), scm);
                    return letrec_scm(new_bindings, Symbol (tmp));
                }
                return scm;
            }
            Prim1 (op, box e) => prim1_scm(op, self.remove(e, true)),
            Prim2 (op, box e1, box e2) => prim2_scm(op, self.remove(e1, true), self.remove(e2, true)),
            Prim3 (op, box e1, box e2, box e3) => 
                prim3_scm(op, self.remove(e1, true), self.remove(e2, true), self.remove(e3, true)),
            Symbol (s) => Symbol (s),
            Quote (box imm) => quote_scm(imm),
            Void => Void,
            other => panic!("Invalid Program {}", other),
        }
    }
}

pub struct SanitizeBindingForms {}
impl SanitizeBindingForms {
    pub fn run(&self, scm: Scheme) -> Scheme {
        self.sanitize(scm)
    }

    fn sanitize(&self, scm: Scheme) -> Scheme {
        use Scheme::*;
        match scm {
            If (box pred, box b1, box b2) => if2_scm(
                self.sanitize(pred),
                self.sanitize(b1),
                self.sanitize(b2),
            ),
            Begin (mut exprs) => Begin (
                exprs.into_iter().map(|e| self.sanitize(e)).collect()
            ),
            Funcall (box func, mut values) => funcall_scm(
                self.sanitize(func), 
                values.into_iter().map(|e| self.sanitize(e)).collect()
            ),
            Let (mut bindings, box body) => {
                let mut let_bindings = HashMap::new();
                let mut letrec_bindings = HashMap::new();
                for (k, v) in bindings.drain() {
                    let v = self.sanitize(v);
                    if let Lambda (_args, _body) = &v {
                        letrec_bindings.insert(k, v);
                    } else {
                        let_bindings.insert(k, v);
                    }
                }
                let body = self.sanitize(body);
                if let_bindings.is_empty() && letrec_bindings.is_empty() { return body; }
                if letrec_bindings.is_empty() { return let_scm(let_bindings, body); }
                if let_bindings.is_empty() { return letrec_scm(letrec_bindings, body); }
                return let_scm(let_bindings, letrec_scm(letrec_bindings, body));
            }
            Letrec (mut bindings, box body) => {
                let mut new_bindings = HashMap::new();
                for (k, v) in bindings.drain() {
                    new_bindings.insert(k, self.sanitize(v));
                }
                return letrec_scm(new_bindings, self.sanitize(body));
            }
            Lambda (args, box body) => lambda_scm(args, self.sanitize(body)),
            Prim1 (op, box e) => prim1_scm(op, self.sanitize(e)),
            Prim2 (op, box e1, box e2) => prim2_scm(op, self.sanitize(e1), self.sanitize(e2)),
            Prim3 (op, box e1, box e2, box e3) => prim3_scm(op, self.sanitize(e1), self.sanitize(e2), self.sanitize(e3)),
            Symbol (s) => Symbol (s),
            Quote (box imm) => quote_scm(imm),
            Void => Void,
            other => panic!("Invalid Program {}", other),
        }
    }
}

pub struct UncoverFree {}
impl UncoverFree {
    pub fn run(&self, scm: Scheme) -> Scheme {
        let (fset, scm) = self.uncover_free(scm);
        return scm;
    }

    fn uncover_free(&self, scm: Scheme) -> (HashSet<String>, Scheme) {
        use Scheme::*;
        match scm {
            Symbol (s) => {
                let mut free = HashSet::new();
                free.insert(s.clone());
                return (free, Symbol (s));
            }
            Quote (box imm) => (HashSet::new(), quote_scm(imm)),
            Void => (HashSet::new(), Void),
            If (box pred, box b1, box b2) => {
                let (pf, pred) = self.uncover_free(pred);
                let (bf1, b1) = self.uncover_free(b1);
                let (bf2, b2) = self.uncover_free(b2);
                let new_set = self.union_freeset(vec![pf, bf1, bf2]);
                return (new_set, if2_scm(pred, b1, b2));
            }
            Begin (mut exprs) => {
                let mut sets = vec![];
                let mut new_exprs = vec![];
                for e in exprs.into_iter() {
                    let (fset, e) = self.uncover_free(e);
                    sets.push(fset);
                    new_exprs.push(e);
                }
                let new_set = self.union_freeset(sets);
                return (new_set, Begin (new_exprs));
            }
            Let (mut bindings, box e) => {
                let mut new_bindings = HashMap::new();
                let mut sets = vec![];
                for (k, v) in bindings.drain() {
                    let (fset, v) = self.uncover_free(v);
                    sets.push(fset);
                    new_bindings.insert(k, v);
                }
                let (fset, e) = self.uncover_free(e);
                sets.push(fset);
                let mut new_set = self.union_freeset(sets);
                for k in new_bindings.keys() {
                    new_set.remove(k);
                }
                return (new_set, let_scm(new_bindings, e));
            }
            Letrec (mut lambdas, box e) => {
                let mut new_bindings = HashMap::new();
                let mut sets = vec![];
                for (k, v) in lambdas.drain() {
                    let (fset, v) = self.uncover_free(v);
                    sets.push(fset);
                    new_bindings.insert(k, v);
                }
                let (fset, e) = self.uncover_free(e);
                sets.push(fset);
                let mut new_set = self.union_freeset(sets);
                for k in new_bindings.keys() {
                    new_set.remove(k);
                }
                return (new_set, letrec_scm(new_bindings, e));
            }
            Prim1 (op, box e) => {
                let (fset, e) = self.uncover_free(e);
                return (fset, prim1_scm(op, e));
            }
            Prim2 (op, box e1, box e2) => {
                let (fset1, e1) = self.uncover_free(e1);
                let (fset2, e2) = self.uncover_free(e2);
                let new_set = self.union_freeset(vec![fset1, fset2]);
                return (new_set, prim2_scm(op, e1, e2));
            }
            Prim3 (op, box e1, box e2, box e3) => {
                let (fset1, e1) = self.uncover_free(e1);
                let (fset2, e2) = self.uncover_free(e2);
                let (fset3, e3) = self.uncover_free(e3);
                let new_set = self.union_freeset(vec![fset1, fset2, fset3]);
                return (new_set, prim3_scm(op, e1, e2, e3));
            }
            Funcall (box func, mut args) => {
                let (fset1, func) = self.uncover_free(func);
                let mut sets = vec![fset1];
                args = args.into_iter().map(|a| {
                    let (fset, a) = self.uncover_free(a);
                    sets.push(fset);
                    a
                }).collect();
                let new_set = self.union_freeset(sets);
                return (new_set, funcall_scm(func, args));
            }
            Lambda (args, box body) => {
                let (mut fset, body) = self.uncover_free(body);
                for a in args.iter() {
                    fset.remove(a);
                }
                let freevars: Vec<_> = fset.iter().map(|e| e.to_string()).collect();
                let free = Free (freevars, Box::new(body));
                return (fset, lambda_scm(args, free));
            }
            e => panic!("Invalid Program {}", e),
        }
    }

    fn union_freeset(&self, sets: Vec<HashSet<String>>) -> HashSet<String> {
        let mut new_set = HashSet::new();
        for mut set in sets {
            for e in set.drain() {
                new_set.insert(e);
            }
        }
        return new_set;
    }
}


pub struct ConvertClosure {}
impl ConvertClosure {
    pub fn run(&self, scm: Scheme) -> Scheme {
        self.convert_closure(scm)
    }

    fn convert_closure(&self, scm: Scheme) -> Scheme {
        use Scheme::*;
        match scm {
            Symbol (s) => Symbol (s), 
            Quote (box imm) => quote_scm(imm),
            Void => Void,
            If (box pred, box b1, box b2) => {
                let new_pred = self.convert_closure(pred);
                let new_b1 = self.convert_closure(b1);
                let new_b2 = self.convert_closure(b2);
                return if2_scm(new_pred, new_b1, new_b2);
            }
            Begin (mut exprs) => {
                exprs = exprs.into_iter().map(|e| self.convert_closure(e)).collect();
                return Begin (exprs);
            }
            Let (mut bindings, box value) => {
                let mut new_bindings = HashMap::new();
                for (k, v) in bindings.into_iter() {
                    new_bindings.insert(k, self.convert_closure(v));
                }
                return let_scm(new_bindings, self.convert_closure(value));
            }
            Letrec (mut bindings, box value) => {
                let mut new_bindings = HashMap::new();
                let mut clos = vec![];
                for (k, v) in bindings.drain() {
                    if let Lambda (mut args, box Free (mut fvars, box body)) = v {
                        let label = uvar_to_label(&k);
                        clos.push((k.clone(), label.clone(), fvars.clone()));   // prepare closures
                        let new_body = self.convert_closure(body);
                        args.push(k.clone());                                   // cp as argument
                        fvars.push(k);                                          // cp into bind-free form
                        let new_lambda = lambda_scm(args, Bindfree (fvars, Box::new(new_body)));
                        new_bindings.insert(label, new_lambda);
                    } else {
                        unreachable!();
                    }
                }
                // here, lambdas are ready and closures is ready too.
                let new_value = self.convert_closure(value);
                let closures = Closures (clos, Box::new(new_value));
                return letrec_scm(new_bindings, closures);
            }
            Prim1 (op, box e) => {
                let e = self.convert_closure(e);
                return prim1_scm(op, e);
            }
            Prim2 (op, box e1, box e2) => {
                let e1 = self.convert_closure(e1);
                let e2 = self.convert_closure(e2);
                return prim2_scm(op, e1, e2);
            }
            Prim3 (op, box e1, box e2, box e3) => {
                let e1 = self.convert_closure(e1);
                let e2 = self.convert_closure(e2);
                let e3 = self.convert_closure(e3);
                return prim3_scm(op, e1, e2, e3);
            }
            Funcall (box func, mut args) => {
                args = args.into_iter().map(|x| self.convert_closure(x)).collect();
                // I choose to add cp as the last argument
                if let Symbol (s) = &func {
                    args.push(Symbol (s.to_string())); 
                    return funcall_scm(func, args);
                } 
                // func is a complex expression
                let tmp = gen_uvar();
                let mut new_bindings = HashMap::new();
                new_bindings.insert(tmp.clone(), self.convert_closure(func));
                args.push(Symbol (tmp.clone()));
                return let_scm(new_bindings, funcall_scm(Symbol (tmp), args));
            }
            e => panic!("Invalid Program {}", e),
        }
    }
}

pub struct OptimizeKnownCall {}
impl OptimizeKnownCall {
    pub fn run(&self, scm: Scheme) -> Scheme {
        let mut mapping = HashMap::new();
        self.optimize(scm, &mut mapping)
    }

    fn optimize(&self, scm: Scheme, mapping: &mut HashMap<String, String>) -> Scheme {
        use Scheme::*;
        match scm {
            Symbol (s) => Symbol (s),
            Quote (box imm) => quote_scm(imm),
            Void => Void,
            If (box pred, box b1, box b2) => {
                let new_pred = self.optimize(pred, mapping);
                let new_b1 = self.optimize(b1, mapping);
                let new_b2 = self.optimize(b2, mapping);
                return if2_scm(new_pred, new_b1, new_b2);
            }
            Begin (mut exprs) => {
                exprs = exprs.into_iter().map(|e| self.optimize(e, mapping)).collect();
                return Begin (exprs);
            }
            Let (mut bindings, box value) => {
                let mut new_bindings = HashMap::new();
                for (k, val) in bindings.drain() {
                    new_bindings.insert(k, self.optimize(val, mapping));
                }
                return let_scm(new_bindings, self.optimize(value, mapping));
            }
            Letrec (mut bindings, box clos) => {
                // here, we should collects closures firstly. or we will lost some optimization.
                let clos = self.optimize(clos, mapping);
                let mut new_bindings = HashMap::new();
                for (k, val) in bindings.drain() {
                    new_bindings.insert(k, self.optimize(val, mapping));
                }
                return letrec_scm(new_bindings, clos);
            }
            Lambda (args, box Bindfree (mut new_fvars, box body)) => {
                let new_body = self.optimize(body, mapping);
                return lambda_scm(args, Bindfree (new_fvars, Box::new(new_body)));
            }
            // we collect mapping here 
            Closures (clos, box body) => {
                for (cp, code, fvars) in &clos {
                    mapping.insert(cp.to_string(), code.to_string());
                }     
                return Closures (clos, Box::new(self.optimize(body, mapping)));
            }
            Prim1 (op, box e) => prim1_scm(op, self.optimize(e, mapping)),
            Prim2 (op, box e1, box e2) => prim2_scm(op, self.optimize(e1, mapping), self.optimize(e2, mapping)),
            Prim3 (op, box e1, box e2, box e3) => prim3_scm(op, self.optimize(e1, mapping), self.optimize(e2, mapping), self.optimize(e3, mapping)),
            // perform replace here
            Funcall (box Symbol (mut func), mut args) => {
                // since variables is unique, perform args here will not effect its result.
                args = args.into_iter().map(|e| self.optimize(e, mapping)).collect();
                match mapping.get(&func) {
                    Some (labl) => funcall_scm(Symbol (labl.to_string()), args),
                    None => funcall_scm(Symbol (func), args),
                }
            }
            e => panic!("Invalid Program {}", e),
        }
    }
}

pub struct IntroduceProceduraPrimitives {}
impl IntroduceProceduraPrimitives {
    pub fn run(&self, scm: Scheme) -> Scheme {
        return self.intro(scm, "", &vec![]);
    }

    fn intro(&self, scm: Scheme, cp: &str, fvars: &Vec<String>) -> Scheme {
        use Scheme::*;
        match scm {
            Symbol (s) => {
                if fvars.contains(&s) {
                    let index = self.find_freevar_index(fvars, s.as_str());
                    return prim2_scm("procedure-ref".to_string(), Symbol (cp.to_string()), quote_scm(Int64 (index)));
                }
                return Symbol (s);
            }
            Quote (box imm) => quote_scm(imm),
            Void => Void,
            If (box pred, box b1, box b2) => {
                let new_pred = self.intro(pred, cp, fvars);
                let new_b1 = self.intro(b1, cp, fvars);
                let new_b2 = self.intro(b2, cp, fvars);
                return if2_scm(new_pred, new_b1, new_b2);
            }
            Begin (mut exprs) => {
                exprs = exprs.into_iter().map(|e| self.intro(e, cp, fvars)).collect();
                return Begin (exprs);
            }
            Let (mut bindings, box value) => {
                let mut new_bindings = HashMap::new();
                for (k, val) in bindings.drain() {
                    new_bindings.insert(k, self.intro(val, cp, fvars));
                }
                return let_scm(new_bindings, self.intro(value, cp, fvars));
            }
            // letrec deconstruct into Lambda and Closures as follow
            Letrec (mut bindings, box clos) => {
                let mut new_bindings = HashMap::new();
                for (k, val) in bindings.drain() {
                    new_bindings.insert(k, self.intro(val, cp, fvars));
                }
                return letrec_scm(new_bindings, self.intro(clos, cp, fvars));
            }
            // here, we using new fvars and cp, because lambda body is closed.
            Lambda (args, box Bindfree (mut new_fvars, box body)) => {
                let new_cp = new_fvars.pop().unwrap();
                let new_body = self.intro(body, &new_cp, &new_fvars);
                return lambda_scm(args, new_body);
            }
            // separate it from letrec to show its self-contained.
            // BE CAREFUL: there are two cp, fvars. 
            Closures (clos, box body) => {
                let mut bindings = HashMap::new();
                let mut exprs = vec![];
                for (clos_cp, clos_code, clos_fvars) in clos {
                    let length = clos_fvars.len();
                    let alloc = prim2_scm("make-procedure".to_string(), Symbol (clos_code), quote_scm(Int64 (length as i64)));
                    for (i, clos_fvar) in clos_fvars.into_iter().enumerate() {
                        // check if such a clos_fvar is yet another fvar in fvars
                        let var = if fvars.contains(&clos_fvar) { 
                            let index = self.find_freevar_index(fvars, &clos_fvar);
                            prim2_scm("procedure-ref".to_string(), Symbol (cp.to_string()), quote_scm(Int64 (index)))
                        } else { 
                            Symbol (clos_fvar) 
                        };
                        exprs.push(prim3_scm("procedure-set!".to_string(), Symbol (clos_cp.clone()),  quote_scm(Int64 (i as i64)), var));
                    }
                    bindings.insert(clos_cp, alloc);
                }     
                exprs.push(self.intro(body, cp, fvars));
                return let_scm(bindings, Begin (exprs));
            }
            Prim1 (op, box e) => prim1_scm(op, self.intro(e, cp, fvars)),
            Prim2 (op, box e1, box e2) => prim2_scm(op, self.intro(e1, cp, fvars), self.intro(e2, cp, fvars)),
            Prim3 (op, box e1, box e2, box e3) => prim3_scm(op, self.intro(e1, cp, fvars), self.intro(e2, cp, fvars), self.intro(e3, cp, fvars)),
            Funcall (box Symbol (func), mut args) => {
                args = args.into_iter().map(|e| self.intro(e, cp, fvars)).collect();
                // because we have convert_closure, func must be a symbol  
                // but it is a uvar or a cp or a label?
                if fvars.contains(&func) {
                    let index = self.find_freevar_index(fvars, func.as_str());
                    let proc = prim2_scm("procedure-ref".to_string(), Symbol (cp.to_string()), quote_scm(Int64 (index)));
                    let newfn = prim1_scm("procedure-code".to_string(), proc);
                    return funcall_scm(newfn, args);
                }
                if is_uvar(&func) { 
                    let newfn = prim1_scm("procedure-code".to_string(), Symbol (func));
                    return funcall_scm(newfn, args);
                }
                if is_label(&func) { 
                    return funcall_scm(Symbol (func), args);
                }
                panic!("Invalid procedure {}", func);
            }
            e => panic!("Invalid Program {}", e),
        }
    }
    
    fn find_freevar_index(&self, fvars: &Vec<String>, var: &str) -> i64 {
        let index: i64 = fvars.iter().position(|x| x == var).unwrap() as i64;
        return index;
    }
}

pub struct LiftLetrec {}
impl LiftLetrec {
    pub fn run(&self, scm: Scheme) -> Scheme {
        use Scheme::*;
        let mut lambdas = HashMap::new();
        let body = self.lift_letrec(scm, &mut lambdas);
        return Letrec (lambdas, Box::new(body));
    }

    fn lift_letrec(&self, scm: Scheme, lambdas: &mut HashMap<String, Scheme>) -> Scheme {
        use Scheme::*;
        match scm {
            If (box pred, box b1, box b2) => {
                let new_pred = self.lift_letrec(pred, lambdas);
                let new_b1 = self.lift_letrec(b1, lambdas);
                let new_b2 = self.lift_letrec(b2, lambdas);
                return if2_scm(new_pred, new_b1, new_b2);
            }
            Begin (mut exprs) => {
                exprs = exprs.into_iter().map(|e| self.lift_letrec(e, lambdas)).collect();
                return Begin (exprs);
            }
            Let (mut bindings, box tail) => {
                let mut new_bindings = HashMap::new();
                for (k, val) in bindings.drain() {
                    new_bindings.insert(k, self.lift_letrec(val, lambdas));
                }
                return let_scm(new_bindings, self.lift_letrec(tail, lambdas));
            }
            Prim1 (op, box e) => prim1_scm(op, self.lift_letrec(e, lambdas)),
            Prim2 (op, box e1, box e2) => {
                return prim2_scm(op, self.lift_letrec(e1, lambdas), self.lift_letrec(e2, lambdas));
            }
            Prim3 (op, box e1, box e2, box e3) => {
                return prim3_scm(op, self.lift_letrec(e1, lambdas), self.lift_letrec(e2, lambdas), self.lift_letrec(e3, lambdas));
            }
            Funcall (box func, mut args) => {
                let new_func = self.lift_letrec(func, lambdas);
                args = args.into_iter().map(|e| self.lift_letrec(e, lambdas)).collect();
                return funcall_scm(new_func, args);
            }
            Letrec (mut bindings, box body) => {
                for (k, val) in bindings.drain() {
                    let new_val = self.lift_letrec(val, lambdas);
                    lambdas.insert(k, new_val);
                }
                return self.lift_letrec(body, lambdas);
            }
            Lambda (args, box body) => Lambda (args, Box::new(self.lift_letrec(body, lambdas))),
            Quote (box imm) => Quote (Box::new(imm)),
            Symbol (s) => Symbol (s),
            Void => Void,
            other => panic!("Invalid Scheme Program {}", other),
        }
    }
}

pub struct NormalizeContext {}
impl NormalizeContext {
    pub fn run(&self, scm: Scheme) -> Scheme {
        use Scheme::*;
        match scm {
            Letrec (mut lambdas, box value) => {
                let mut new_bindings = HashMap::new();
                for (k, v) in lambdas.drain() {
                    new_bindings.insert(k, self.value_helper(v));
                }
                return letrec_scm(new_bindings, self.value_helper(value));
            }
            other => panic!("Invalid Scheme Program {}", other), 
        }
    }

    fn value_helper(&self, value: Scheme) -> Scheme {
        use Scheme::*;
        match value {
            If (box pred, box b1, box b2) => {
                let new_pred = self.pred_helper(pred);
                let new_b1 = self.value_helper(b1);
                let new_b2 = self.value_helper(b2);
                return if2_scm(new_pred, new_b1, new_b2);
            }
            Begin (mut exprs) => {
                let value = exprs.pop().unwrap();
                exprs = exprs.into_iter().map(|e| self.effect_helper(e)).collect();
                exprs.push(self.value_helper(value));
                return make_nopless_begin(exprs);
            }
            Let (mut bindings, box tail) => {
                let mut new_bindings = HashMap::new();
                for (k, val) in bindings.drain() {
                    new_bindings.insert(k, self.value_helper(val));
                }
                return let_scm(new_bindings, self.value_helper(tail));
            }
            Lambda (args, box body) => {
                let new_body = self.value_helper(body);
                return lambda_scm(args, new_body);
            }
            Prim1 (op, box e) if is_value_prim(op.as_str()) => prim1_scm(op, self.value_helper(e)),
            Prim2 (op, box e1, box e2) if is_value_prim(op.as_str()) => prim2_scm(op, self.value_helper(e1), self.value_helper(e2)),
            Prim3 (op, box e1, box e2, box e3) if is_value_prim(op.as_str()) => prim3_scm(op, self.value_helper(e1), self.value_helper(e2), self.value_helper(e3)),
            Prim1 (op, box e) if is_pred_prim(op.as_str()) => {
                let e = prim1_scm(op, self.value_helper(e));
                return if2_scm(e, quote_scm(Bool (true)), quote_scm(Bool (false)));
            }
            Prim2 (op, box e1, box e2) if is_pred_prim(op.as_str()) => {
                let e = prim2_scm(op, self.value_helper(e1), self.value_helper(e2));
                return if2_scm(e, quote_scm(Bool (true)), quote_scm(Bool (false)));
            }
            Prim3 (op, box e1, box e2, box e3) if is_pred_prim(op.as_str()) => {
                let e = prim3_scm(op, self.value_helper(e1), self.value_helper(e2), self.value_helper(e3));
                return if2_scm(e, quote_scm(Bool (true)), quote_scm(Bool (false)));
            }
            Prim1 (op, box e) if is_effect_prim(op.as_str()) => {
                let e = prim1_scm(op, self.value_helper(e));
                return Begin (vec![e, Void]);
            }
            Prim2 (op, box e1, box e2) if is_effect_prim(op.as_str()) => {
                let e = prim2_scm(op, self.value_helper(e1), self.value_helper(e2));
                return Begin (vec![e, Void]);
            }
            Prim3 (op, box e1, box e2, box e3) if is_effect_prim(op.as_str()) => {
                let e = prim3_scm(op, self.value_helper(e1), self.value_helper(e2), self.value_helper(e3));
                return Begin (vec![e, Void]);
            }
            Funcall (box func, mut args) => {
                let new_func = self.value_helper(func);
                args = args.into_iter().map(|e| self.value_helper(e)).collect();
                return funcall_scm(new_func, args);
            }
            Quote (box imm) => Quote (Box::new(imm)),
            Symbol (s) => Symbol (s),
            Void => Void,
            other => panic!("Invalid Value {}", other),
        }
    }

    fn pred_helper(&self, pred: Scheme) -> Scheme {
        use Scheme::*;
        match pred {
            If (box pred, box b1, box b2) => {
                let new_pred = self.pred_helper(pred);
                let new_b1 = self.pred_helper(b1);
                let new_b2 = self.pred_helper(b2);
                return if2_scm(new_pred, new_b1, new_b2);
            }
            Begin (mut exprs) => {
                let pred = exprs.pop().unwrap();
                exprs = exprs.into_iter().map(|e| self.effect_helper(e)).collect();
                exprs.push(self.pred_helper(pred));
                return make_nopless_begin(exprs);
            }
            Let (mut bindings, box pred) => {
                let mut new_bindings = HashMap::new();
                for (k, val) in bindings.drain() {
                    new_bindings.insert(k, self.value_helper(val));
                }
                return let_scm(new_bindings, self.pred_helper(pred));
            }
            Prim1 (op, box e) if is_pred_prim(op.as_str()) => prim1_scm(op, self.value_helper(e)),
            Prim2 (op, box e1, box e2) if is_pred_prim(op.as_str()) => prim2_scm(op, self.value_helper(e1), self.value_helper(e2)),
            Prim3 (op, box e1, box e2, box e3) if is_pred_prim(op.as_str()) => prim3_scm(op, self.value_helper(e1), self.value_helper(e2), self.value_helper(e3)),
            Prim1 (op, box e) if is_value_prim(op.as_str()) => {
                let e = prim1_scm(op, self.value_helper(e));
                let relop = prim2_scm("eq?".to_string(), e, quote_scm(Bool (false)));
                return if2_scm(relop, Bool (false), Bool (true));
            }
            Prim2 (op, box e1, box e2) if is_value_prim(op.as_str()) => {
                let e = prim2_scm(op, self.value_helper(e1), self.value_helper(e2));
                let relop = prim2_scm("eq?".to_string(), e, quote_scm(Bool (false)));
                return if2_scm(relop, Bool (false), Bool (true));
            }
            Prim3 (op, box e1, box e2, box e3) if is_value_prim(op.as_str()) => {
                let e = prim3_scm(op, self.value_helper(e1), self.value_helper(e2), self.value_helper(e3));
                let relop = prim2_scm("eq?".to_string(), e, quote_scm(Bool (false)));
                return if2_scm(relop, Bool (false), Bool (true));
            }
            Prim1 (op, box e) if is_effect_prim(op.as_str()) => {
                let e = prim1_scm(op, self.value_helper(e));
                return Begin (vec![e, Bool (true)]);
            }
            Prim2 (op, box e1, box e2) if is_effect_prim(op.as_str()) => {
                let e = prim2_scm(op, self.value_helper(e1), self.value_helper(e2));
                return Begin (vec![e, Bool (true)]);
            }
            Prim3 (op, box e1, box e2, box e3) if is_effect_prim(op.as_str()) => {
                let e = prim3_scm(op, self.value_helper(e1), self.value_helper(e2), self.value_helper(e3));
                return Begin (vec![e, Bool (true)]);
            }
            Funcall (box func, mut args) => {
                let new_func = self.value_helper(func);
                args = args.into_iter().map(|e| self.value_helper(e)).collect();
                let e = funcall_scm(new_func, args);
                let relop = prim2_scm("eq?".to_string(), e, quote_scm(Bool (false)));
                return if2_scm(relop, Bool (false), Bool (true));
            }
            Quote (box Bool (b)) => Bool (b),
            // note that the EmptyList is convert to (true). Because anything if is not #f is (true)
            Quote (box other) => Bool (true),
            // the same reason as above
            Void => Bool (true), 
            // is label comparable?
            Symbol (s) => {
                let relop = prim2_scm("eq?".to_string(), Symbol (s), quote_scm(Bool (false)));
                return if2_scm(relop, Bool (false), Bool (true));
            }
            other => panic!("Invalid predicate {}", other),
        }
    }

    fn effect_helper(&self, effect: Scheme) -> Scheme {
        use Scheme::*;
        match effect {
            If (box pred, box b1, box b2) => {
                let new_pred = self.pred_helper(pred);
                let new_b1 = self.effect_helper(b1);
                let new_b2 = self.effect_helper(b2);
                return if2_scm(new_pred, new_b1, new_b2);
            }
            Begin (mut exprs) => {
                exprs = exprs.into_iter().map(|e| self.effect_helper(e)).collect();
                return make_nopless_begin(exprs);
            }
            Let (mut bindings, box tail) => {
                let mut new_bindings = HashMap::new();
                for (k, val) in bindings.drain() {
                    new_bindings.insert(k, self.value_helper(val));
                }
                return let_scm(new_bindings, self.effect_helper(tail));
            }
            // effect group
            Prim1 (op, box e) if is_effect_prim(op.as_str()) => prim1_scm(op, self.value_helper(e)),
            Prim2 (op, box e1, box e2) if is_effect_prim(op.as_str()) => prim2_scm(op, self.value_helper(e1), self.value_helper(e2)),
            Prim3 (op, box e1, box e2, box e3) if is_effect_prim(op.as_str()) => prim3_scm(op, self.value_helper(e1), self.value_helper(e2), self.value_helper(e3)),
            // no-effect group, evaluate its args for effection if any
            Prim1 (op, box e) => self.effect_helper(e),
            Prim2 (op, box e1, box e2)  => {
                let exprs = vec![
                    self.effect_helper(e1),
                    self.effect_helper(e2),
                ];
                return make_nopless_begin(exprs);
            }
            Prim3 (op, box e1, box e2, box e3) => {
                let exprs = vec![
                    self.effect_helper(e1),
                    self.effect_helper(e2),
                    self.effect_helper(e3),
                ];
                return make_nopless_begin(exprs);
            }
            Funcall (box func, mut args) => {
                let new_func = self.value_helper(func);
                args = args.into_iter().map(|e| self.value_helper(e)).collect();
                return funcall_scm(new_func, args);
            }
            Quote (box imm) => Nop,
            Symbol (s) => Nop,
            Void => Nop,
            other => panic!("Invalid Effect {}", other),
        }
    }
}

pub struct SpecifyRepresentation {}
impl SpecifyRepresentation {
    pub fn run(&self, scm: Scheme) -> Scheme {
        use Scheme::*;
        match scm {
            Letrec (mut lambdas, box value) => {
                let mut new_bindings = HashMap::new();
                for (k, v) in lambdas.drain() {
                    new_bindings.insert(k, self.value_helper(v));
                }
                return letrec_scm(new_bindings, self.value_helper(value));
            }
            e => panic!("Invalid Program {}", e),
        }
    }

    fn value_helper(&self, value: Scheme) -> Scheme {
        use Scheme::*;
        match value {
            If (box pred, box b1, box b2) => {
                let new_pred = self.pred_helper(pred);
                let new_b1 = self.value_helper(b1);
                let new_b2 = self.value_helper(b2);
                return if2_scm(new_pred, new_b1, new_b2);
            }
            Begin (mut exprs) => {
                let value = exprs.pop().unwrap();
                exprs = exprs.into_iter().map(|x| self.effect_helper(x)).collect();
                exprs.push(self.value_helper(value));
                return Begin (exprs);
            }
            Let (mut bindings, box value) => {
                let mut new_bindings = HashMap::new();
                for (sym, val) in bindings.drain() {
                    new_bindings.insert(sym, self.value_helper(val));
                }
                return Let (new_bindings, Box::new(self.value_helper(value)));
            }
            Lambda (args, box body) => lambda_scm(args, self.value_helper(body)),
            Funcall (box mut func, mut args) => {
                let new_func = self.value_helper(func);
                args = args.into_iter().map(|a| self.value_helper(a)).collect();
                return Funcall (Box::new(new_func), args);
            }
            Prim1 (op, box Quote (box Int64 (i))) if op.as_str() == "make-vector" => {
                let tmp = gen_uvar();
                let vsize = (i << ALIGN_SHIFT) + DISP_VDATA;
                let ptr = prim2_scm("+".to_string(), Alloc (Box::new(Int64 (vsize))), Int64 (TAG_VECTOR));
                let mut bindings = HashMap::new();
                bindings.insert(tmp.clone(), ptr);
                let exprs = vec![
                    mset_scm(Symbol (tmp.clone()), Int64 (VLEN_OFFSET), Int64 (i << SHIFT_FIXNUM)),
                    Symbol (tmp),
                ];
                return let_scm(bindings, Begin (exprs));
            }
            Prim1 (op, box value) if is_value_prim(op.as_str()) => {
                let new_value = self.value_helper(value);
                match op.as_str() {
                    "car" => mref_scm(new_value, Int64 (CAR_OFFSET)),
                    "cdr" => mref_scm(new_value, Int64 (CDR_OFFSET)),
                    "vector-length" => mref_scm(new_value, Int64 (VLEN_OFFSET)),
                    "procedure-code" => mref_scm(new_value, Int64 (PROC_CODE_OFFSET)),
                    "make-vector" => {
                        let tmp1 = gen_uvar();                            
                        let mut bindings1 = HashMap::new();
                        bindings1.insert(tmp1.clone(), new_value);
                        let tmp2 = gen_uvar();
                        let vsize = prim2_scm("+".to_string(), Int64 (DISP_VDATA), Symbol (tmp1.clone()));
                        let ptr = prim2_scm("+".to_string(), Alloc (Box::new(vsize)), Int64 (TAG_VECTOR));
                        let mut bindings2 = HashMap::new();
                        bindings2.insert(tmp2.clone(), ptr);
                        let exprs = vec![
                            mset_scm(Symbol (tmp2.clone()), Int64 (VLEN_OFFSET), Symbol (tmp1)),
                            Symbol (tmp2),
                        ];
                        return let_scm(bindings1, let_scm(bindings2, Begin (exprs)));
                    }
                    other => Prim1 (op, Box::new(new_value))
                }
            }
            Prim2 (op, box Quote (box Int64 (i)), box e) | Prim2 (op, box e, box Quote (box Int64 (i))) if op.as_str() == "*" => {
                let new_e = self.value_helper(e); 
                let new_i = Int64 (i);
                return prim2_scm(op, new_e, new_i);
            }
            Prim2 (op, box labl, box Quote (box Int64 (i))) if op.as_str() == "make-procedure" => {
                let tmp = gen_uvar();
                let vsize = (i << ALIGN_SHIFT) + DISP_PDATA;
                let ptr = prim2_scm("+".to_string(), Alloc (Box::new(Int64 (vsize))), Int64 (TAG_PROC));
                let mut bindings = HashMap::new();
                bindings.insert(tmp.clone(), ptr);
                let exprs = vec![
                    mset_scm(Symbol (tmp.clone()), Int64 (PROC_CODE_OFFSET), labl),
                    Symbol (tmp),
                ];
                return let_scm(bindings, Begin (exprs));
            } 
            Prim2 (op, box e, box Quote (box Int64 (i))) if op.as_str() == "vector-ref" || op.as_str() == "procedure-ref" => {
                let offset = match op.as_str() {
                    "vector-ref" => VDATA_OFFSET,
                    "procedure-ref" => PROC_DATA_OFFSET,
                    other => panic!("Invalid prim2 {}", other),
                };
                let new_e = self.value_helper(e); 
                let n = (i << ALIGN_SHIFT) + offset;
                return mref_scm(new_e, Int64(n));
            }
            Prim2 (op, box v1, box v2) if is_value_prim(op.as_str()) => {
                let new_v1 = self.value_helper(v1);
                let new_v2 = self.value_helper(v2);
                match op.as_str() {
                    "*" => {
                        let new_v2 = prim2_scm("sra".to_string(), new_v2, Int64 (SHIFT_FIXNUM as i64));
                        return prim2_scm(op, new_v1, new_v2);
                    }
                    "vector-ref" => {
                        let new_v2 = prim2_scm("+".to_string(), new_v2, Int64 (VDATA_OFFSET));
                        return mref_scm(new_v1, new_v2);
                    }
                    "cons" => {
                        let tmp_car = gen_uvar();
                        let tmp_cdr = gen_uvar();
                        let mut bindings = HashMap::new();
                        bindings.insert(tmp_car.clone(), new_v1);
                        bindings.insert(tmp_cdr.clone(), new_v2);
                        let mut bindings_ptr = HashMap::new();
                        let tmp = gen_uvar();
                        let ptr = prim2_scm("+".to_string(), Alloc (Box::new(Int64 (SIZE_PAIR))), Int64 (TAG_PAIR));
                        bindings_ptr.insert(tmp.clone(), ptr);
                        let exprs = vec![
                            mset_scm(Symbol (tmp.clone()), Int64 (CAR_OFFSET), Symbol (tmp_car)),
                            mset_scm(Symbol (tmp.clone()), Int64 (CDR_OFFSET), Symbol (tmp_cdr)),
                            Symbol (tmp),
                        ];
                        return let_scm(bindings, let_scm(bindings_ptr, Begin (exprs)));
                    }
                    other => prim2_scm(op, new_v1, new_v2),
                }
            }
            Quote (box imm) => self.imm_helper(imm),
            Void => Int64 (VOID),
            Symbol (s) => Symbol (s),
            other => panic!("Invalid Scheme Value {}", other),
        }
    }

    fn effect_helper(&self, scm: Scheme) -> Scheme {
        use Scheme::*;
        match scm {
            Nop => Nop,
            If (box pred, box b1, box b2) => {
                let new_pred = self.pred_helper(pred);
                let new_b1 = self.effect_helper(b1);
                let new_b2 = self.effect_helper(b2);
                return if2_scm(new_pred, new_b1, new_b2);
            }
            Begin (mut exprs) => {
                exprs = exprs.into_iter().map(|e| self.effect_helper(e)).collect();
                return Begin(exprs);
            }
            Let (mut bindings, box effect) => {
                let mut new_bindings = HashMap::new();
                for (sym, val) in bindings.drain() {
                    new_bindings.insert(sym, self.value_helper(val));
                }
                return Let (new_bindings, Box::new(self.effect_helper(effect)));
            }
            Prim2 (op, box v1, box v2) if is_effect_prim(op.as_str()) => {
                let new_v1 = self.value_helper(v1);
                let new_v2 = self.value_helper(v2);
                match op.as_str() {
                    "set-car!" => mset_scm(new_v1, Int64 (CAR_OFFSET), new_v2),
                    "set-cdr!" => mset_scm(new_v1, Int64 (CDR_OFFSET), new_v2),
                    other => prim2_scm(op, new_v1, new_v2),
                }
            }
            Prim3 (op, box v1, box Quote (box Int64 (i)), box v3)  if op.as_str() == "vector-set!" || op.as_str() == "procedure-set!" => {
                let offset = match op.as_str() {
                    "vector-set!" => VDATA_OFFSET,
                    "procedure-set!" => PROC_DATA_OFFSET,
                    other => panic!("Invalid prim2 op {}", other),
                };
                let new_v1 = self.value_helper(v1);
                let new_v3 = self.value_helper(v3);
                let n = (i << ALIGN_SHIFT) + offset;
                return mset_scm(new_v1, Int64 (n), new_v3);
            }
            Prim3 (op, box v1, box v2, box v3) if is_effect_prim(op.as_str()) => {
                let new_v1 = self.value_helper(v1);
                let new_v2 = self.value_helper(v2);
                let new_v3 = self.value_helper(v3);
                match op.as_str() {
                    "vector-set!" => {
                        let new_v2 = prim2_scm("+".to_string(), new_v2, Int64 (VDATA_OFFSET));
                        return mset_scm(new_v1, new_v2, new_v3);
                    }
                    e => panic!("Invalid op {}", e),
                }
            }
            Funcall (box mut func, mut args) => {
                let new_func = self.value_helper(func);
                args = args.into_iter().map(|a| self.value_helper(a)).collect();
                return Funcall (Box::new(new_func), args);
            }
            other => panic!("Invalid Scheme Effect {}", other),
        }
    }

    fn pred_helper(&self, pred: Scheme) -> Scheme {
        use Scheme::*;
        match pred {
            Bool (b) => Bool (b),
            If (box pred, box b1, box b2) => {
                let new_pred = self.pred_helper(pred);
                let new_b1 = self.pred_helper(b1);
                let new_b2 = self.pred_helper(b2);
                return if2_scm(new_pred, new_b1, new_b2);
            }
            Begin (mut exprs) => {
                let pred = exprs.pop().unwrap();
                exprs = exprs.into_iter().map(|e| self.effect_helper(e)).collect();
                exprs.push(self.pred_helper(pred));
                return Begin(exprs);
            }
            Let (mut bindings, box pred) => {
                let mut new_bindings = HashMap::new();
                for (sym, val) in bindings.drain() {
                    new_bindings.insert(sym, self.value_helper(val));
                }
                return Let (new_bindings, Box::new(self.pred_helper(pred)));
            }
            Prim1 (op, box e1) if is_pred_prim(op.as_str()) => {
                let new_e1 = self.value_helper(e1);
                match op.as_str() {
                    "boolean?" => {
                        prim2_scm("=".to_string(), prim2_scm("logand".to_string(), new_e1, Int64 (MASK_BOOL)), Int64 (TAG_BOOL))
                    }
                    "fixnum?" => {
                        prim2_scm("=".to_string(), prim2_scm("logand".to_string(), new_e1, Int64 (MASK_FIXNUM)), Int64 (TAG_FIXNUM))
                    }
                    "pair?" => {
                        prim2_scm("=".to_string(), prim2_scm("logand".to_string(), new_e1, Int64 (MASK_PAIR)), Int64 (TAG_PAIR))
                    }
                    "vector?" => {
                        prim2_scm("=".to_string(), prim2_scm("logand".to_string(), new_e1, Int64 (MASK_VECTOR)), Int64 (TAG_VECTOR))
                    }
                    "null?" => {
                        prim2_scm("=".to_string(), new_e1, Int64 (NIL))
                    }
                    "procedure?" => {
                        prim2_scm("=".to_string(), prim2_scm("logand".to_string(), new_e1, Int64 (MASK_PROC)), Int64 (TAG_PROC))
                    }
                    other => panic!("Invalid Predicate {}", other),
                }
            }
            Prim2 (op, box e1, box e2) if is_pred_prim(op.as_str()) => {
                let new_e1 = self.value_helper(e1);
                let new_e2 = self.value_helper(e2);
                match op.as_str() {
                    "eq?" => prim2_scm("=".to_string(), new_e1, new_e2),
                    other => prim2_scm(op, new_e1, new_e2),
                }
            }
            e => panic!("Invalid Scheme Pred {}", e),
        }
    }

    fn imm_helper(&self, imm: Scheme) -> Scheme {
        use Scheme::*;
        match imm {
            Int64 (i) => Int64 ( i << SHIFT_FIXNUM ),
            EmptyList => Int64 ( NIL ),
            Bool (true) => Int64 ( TRUE ),
            Bool (false) => Int64 ( FALSE ),
            any => panic!("Invalid Immediate {}!", any),
        }
    }
}


pub struct UncoverLocals {}
impl UncoverLocals {
    pub fn run(&self, scm: Scheme) -> Scheme {
        use Scheme::*;
        match scm {
            Letrec (mut lambdas, box value) => {
                let mut new_bindings = HashMap::new();
                for (k, v) in lambdas.drain() {
                    new_bindings.insert(k, self.helper(v));
                }
                return letrec_scm(new_bindings, self.helper(value));
            }
            e => panic!("Invalid Program {}", e),
        }
    }

    fn helper(&self, scm: Scheme) -> Scheme {
        use Scheme::*;
        match scm {
            Lambda (args, box tail) => {
                let body = self.helper(tail);
                Lambda (args, Box::new(body))
            }
            tail => {
                let mut locals = HashSet::new();
                self.tail_helper(&tail, &mut locals);
                Locals (locals, Box::new(tail))
            }
        }
    }

    fn tail_helper(&self, tail: &Scheme, locals: &mut HashSet<String>) {
        use Scheme::*;
        match tail {
            Prim2 (op, box v1, box v2) => {
                self.value_helper(v1, locals);
                self.value_helper(v2, locals);
            }
            Alloc (box value) => self.value_helper(value, locals),
            Funcall (box v, args) => {
                self.value_helper(v, locals);
                args.iter().for_each(|a| self.value_helper(a, locals));
            }
            If (box pred, box b1, box b2) => {
                self.pred_helper(pred, locals);
                self.tail_helper(b1, locals);
                self.tail_helper(b2, locals);
            }
            Begin (exprs) => {
                let exprs_slice = exprs.as_slice();
                let last = exprs.len() - 1;
                for i in 0..last {
                    self.effect_helper(&exprs_slice[i], locals);
                }
                self.tail_helper(&exprs_slice[last], locals);
            }
            Mref (box v1, box v2) => {
                self.value_helper(v1, locals);
                self.value_helper(v2, locals);
            }
            Let (bindings, box tail) => {
                for (k, v) in bindings {
                    locals.insert(k.to_string());
                    self.value_helper(v, locals);
                }
                self.tail_helper(tail, locals);
            }
            triv => (),
        }
    }

    fn effect_helper(&self, effect: &Scheme, locals: &mut HashSet<String>) {
        use Scheme::*;
        match effect {
            Nop => (), 
            Mset (box v1, box v2, box v3) => {
                self.value_helper(v1, locals); 
                self.value_helper(v2, locals);
                self.value_helper(v3, locals);
            }
            Funcall (box v, args) => {
                self.value_helper(v, locals);
                args.iter().for_each(|x| self.value_helper(x, locals));
            }
            If (box pred, box b1, box b2) => {
                self.pred_helper(pred, locals);
                self.effect_helper(b1, locals);
                self.effect_helper(b2, locals);
            }
            Begin (exprs) => {
                exprs.iter().for_each(|x| self.value_helper(x, locals));
            }
            Let (bindings, box effect) => {
                for (k, v) in bindings {
                    locals.insert(k.to_string());
                    self.value_helper(v, locals);
                }
                self.effect_helper(effect, locals);
            }
            e => panic!("Invalid effect expression {}", e),
        }
    }

    fn pred_helper(&self, pred: &Scheme, locals: &mut HashSet<String>) {
        use Scheme::*;
        match pred {
            Bool (b) => (),
            Prim2 (relop, box v1, box v2) => {
                self.value_helper(v1, locals);
                self.value_helper(v2, locals);
            }
            If (box pred, box b1, box b2) => {
                self.pred_helper(pred, locals);
                self.pred_helper(b1, locals);
                self.pred_helper(b2, locals);
            }
            Begin (exprs) => {
                let exprs_slice = exprs.as_slice();
                let last = exprs.len() - 1;
                for i in 0..last {
                    self.effect_helper(&exprs_slice[i], locals);
                }
                self.pred_helper(&exprs_slice[last], locals);
            }
            Let (bindings, box pred) => {
                for (k, v) in bindings {
                    locals.insert(k.to_string());
                    self.value_helper(v, locals);
                }
                self.pred_helper(pred, locals);
            }
            e => panic!("Invalid pred expression {}", e),
        }
    }

    fn value_helper(&self, value: &Scheme, locals: &mut HashSet<String>) {
        use Scheme::*;
        match value {
            Prim2 (op, box v1, box v2) => {
                self.value_helper(v1, locals);
                self.value_helper(v2, locals);
            }
            Mref (box v1, box v2) => {
                self.value_helper(v1, locals);
                self.value_helper(v2, locals);
            }
            Alloc (box v) => self.value_helper(v, locals),
            Funcall (box v, args) => {
                self.value_helper(v, locals);
                args.iter().for_each(|x| self.value_helper(x, locals));
            }
            If (box pred, box b1, box b2) => {
                self.value_helper(pred, locals);
                self.value_helper(b1, locals);
                self.value_helper(b2, locals);
            }
            Begin (exprs) => {
                let exprs_slice = exprs.as_slice();
                let last = exprs.len() - 1;
                for i in 0..last {
                    self.effect_helper(&exprs_slice[i], locals);
                }
                self.value_helper(&exprs_slice[last], locals);
            }
            Let (bindings, box v) => {
                for (k, v) in bindings {
                    locals.insert(k.to_string());
                    self.value_helper(v, locals);
                }
                self.value_helper(v, locals);
            }
            triv => (),
        }
    }
}


pub struct RemoveLet {}
impl RemoveLet {
    pub fn run(&self, scm: Scheme) -> Scheme {
        use Scheme::*;
        match scm {
            Letrec (mut lambdas, box value) => {
                let mut new_bindings = HashMap::new();
                for (k, v) in lambdas.drain() {
                    new_bindings.insert(k, self.helper(v));
                }
                return letrec_scm(new_bindings, self.helper(value));
            }
            e => panic!("Invalid Program {}", e),
        }
    }

    fn helper(&self, scm: Scheme) -> Scheme {
        use Scheme::*;
        match scm {
            Lambda (args, box mut body) => {
                body = self.helper(body);
                Lambda (args, Box::new(body))
            }
            Locals (locals, box mut tail) => {
                tail = self.tail_helper(tail);
                Locals (locals, Box::new(tail))
            }
            e => panic!("Invalid Program {}", e),
        }
    }

    fn tail_helper(&self, tail: Scheme) -> Scheme {
        use Scheme::*;
        match tail {
            Prim2 (op, box v1, box v2) => {
                let new_v1 = self.value_helper(v1);
                let new_v2 = self.value_helper(v2);
                return prim2_scm(op, new_v1, new_v2);
            }
            Mref (box v1, box v2) => {
                let new_v1 = self.value_helper(v1);
                let new_v2 = self.value_helper(v2);
                return Mref (Box::new(new_v1), Box::new(new_v2));
            }
            Alloc (box value) => Alloc (Box::new(self.value_helper(value))),
            Funcall (box v, mut args) => {
                let new_v = self.value_helper(v);
                args = args.into_iter().map(|a| self.value_helper(a)).collect();
                return Funcall (Box::new(new_v), args);
            }
            If (box pred, box b1, box b2) => {
                let new_pred = self.pred_helper(pred);
                let new_b1 = self.tail_helper(b1);
                let new_b2 = self.tail_helper(b2);
                return if2_scm(new_pred, new_b1, new_b2);
            }
            Begin (mut exprs) => {
                let mut tail = exprs.pop().unwrap();
                tail = self.tail_helper(tail);
                exprs = exprs.into_iter().map(|x| self.effect_helper(x)).collect();
                exprs.push(tail);
                return Begin (exprs);
            }
            Let (mut bindings, box mut tail) => {
                let mut exprs = vec![];
                for (s, val) in bindings.drain() {
                    exprs.push( set1_scm(Symbol (s), self.value_helper(val)) );
                }
                tail = self.tail_helper(tail);
                exprs.push(tail);
                return Begin (exprs);
            }
            triv => triv,
        }
    }

    fn pred_helper(&self, pred: Scheme) -> Scheme {
        use Scheme::*;
        match pred {
            Prim2 (relop, box v1, box v2) => {
                let new_v1 = self.value_helper(v1);
                let new_v2 = self.value_helper(v2);
                return prim2_scm(relop, new_v1, new_v2);
            }
            If (box pred, box b1, box b2) => {
                let new_pred = self.pred_helper(pred);
                let new_b1 = self.pred_helper(b1);
                let new_b2 = self.pred_helper(b2);
                return if2_scm(new_pred, new_b1, new_b2);
            }
            Begin (mut exprs) => {
                let mut pred = exprs.pop().unwrap();
                pred = self.pred_helper(pred);
                exprs = exprs.into_iter().map(|x| self.effect_helper(x)).collect();
                exprs.push(pred);
                return Begin (exprs);
            }
            Let (mut bindings, box mut pred) => {
                let mut exprs = vec![];
                for (s, val) in bindings.drain() {
                    exprs.push( set1_scm(Symbol (s), self.value_helper(val)) );
                }
                pred = self.pred_helper(pred);
                exprs.push(pred);
                return Begin (exprs);
            }
            e => e,
        }
    }

    fn effect_helper(&self, effect: Scheme) -> Scheme {
        use Scheme::*;
        match effect {
            Nop => Nop,
            Mset (box v1, box v2, box v3) => {
                let new_v1 = self.value_helper(v1);
                let new_v2 = self.value_helper(v2);
                let new_v3 = self.value_helper(v3);
                return mset_scm(new_v1, new_v2, new_v3);
            }
            Funcall (box v, mut args) => {
                let new_v = self.value_helper(v);
                args = args.into_iter().map(|x| self.value_helper(x)).collect();
                return Funcall (Box::new(new_v), args);
            }
            If (box pred, box b1, box b2) => {
                let new_pred = self.pred_helper(pred);
                let new_b1 = self.effect_helper(b1);
                let new_b2 = self.effect_helper(b2);
                return if2_scm(new_pred, new_b1, new_b2); 
            }
            Begin (mut exprs) => {
                exprs = exprs.into_iter().map(|x| self.value_helper(x)).collect();
                return Begin (exprs);
            }
            Let (mut bindings, box mut effect) => {
                let mut exprs = vec![];
                for (s, val) in bindings.drain() {
                    exprs.push( set1_scm(Symbol (s), self.value_helper(val)) );
                }
                effect = self.effect_helper(effect);
                exprs.push(effect);
                return Begin (exprs);
            }
            e => panic!("Invalid effect expression {}", e),
        }
    }

    fn value_helper(&self, value: Scheme) -> Scheme {
        use Scheme::*;
        match value {
            Prim2 (op, box v1, box v2) => {
                let new_v1 = self.value_helper(v1);
                let new_v2 = self.value_helper(v2);
                return prim2_scm(op, new_v1, new_v2);
            }
            Mref (box base, box offset) => {
                let new_base = self.value_helper(base);
                let new_offset = self.value_helper(offset);
                return Mref (Box::new(new_base), Box::new(new_offset));
            }
            Alloc (box v) => Alloc (Box::new(self.value_helper(v))),
            Funcall (box v, mut args) => {
                let new_v = self.value_helper(v);
                args = args.into_iter().map(|x| self.value_helper(x)).collect();
                return Funcall (Box::new(new_v), args);
            }
            If (box pred, box b1, box b2) => {
                let new_pred = self.value_helper(pred);
                let new_b1 = self.value_helper(b1);
                let new_b2 = self.value_helper(b2);
                return if2_scm(new_pred, new_b1, new_b2);
            }
            Begin (mut exprs) => {
                let mut value = exprs.pop().unwrap();
                value = self.value_helper(value);
                exprs = exprs.into_iter().map(|x| self.effect_helper(x)).collect();
                exprs.push(value);
                return Begin (exprs);
            }
            Let (mut bindings, box mut v) => {
                let mut exprs = vec![];
                for (s, val) in bindings.drain() {
                    exprs.push( set1_scm(Symbol (s), self.value_helper(val)) );
                }
                v = self.value_helper(v);
                exprs.push( v );
                return Begin (exprs);
            }
            triv => triv,
        }
    }

}

// This pass serve as a bridge from Scheme to Expr
// It is just a identical mapping.
pub struct CompileToExpr {}
impl CompileToExpr {
    pub fn run(&self, scm: Scheme) -> Expr {
        match scm {
            Scheme::Letrec (mut lambdas, box mut body) => {
                let new_lambdas: Vec<_> = lambdas.into_iter().map(|(labl, e)| {
                    if let Scheme::Lambda (args, box body) = e {
                        let new_body = self.body_helper(body);
                        return Expr::Lambda (labl, args, Box::new(new_body));
                    }
                    unreachable!();
                }).collect();
                let new_body = self.body_helper(body);
                Expr::Letrec (new_lambdas, Box::new(new_body))
            }
            e => panic!("Invalid Program {}", e)
        }
    }

    fn body_helper(&self, scm: Scheme) -> Expr {
        match scm {
            Scheme::Locals (locals, box tail) => {
                let new_tail = self.tail_helper(tail);
                Expr::Locals (locals, Box::new(new_tail))
            }
            e => panic!("Invalid Program {}", e),
        }
    }

    fn tail_helper(&self, tail: Scheme) -> Expr {
        match tail {
            Scheme::Prim2 (op, box v1, box v2) => {
                let new_v1 = self.value_helper(v1);
                let new_v2 = self.value_helper(v2);
                return Expr::Prim2 (op, Box::new(new_v1), Box::new(new_v2));
            }
            Scheme::Mref (box v1, box v2) => {
                let new_v1 = self.value_helper(v1);
                let new_v2 = self.value_helper(v2);
                return Expr::Mref (Box::new(new_v1), Box::new(new_v2));
            }
            Scheme::Alloc (box value) => Alloc (Box::new(self.value_helper(value))),
            Scheme::Funcall (box v, args) => {
                let new_v = self.value_helper(v);
                let new_args: Vec<_> = args.into_iter().map(|a| self.value_helper(a)).collect();
                return Expr::Funcall (Box::new(new_v), new_args);
            }
            Scheme::If (box pred, box b1, box b2) => {
                let new_pred = self.pred_helper(pred);
                let new_b1 = self.tail_helper(b1);
                let new_b2 = self.tail_helper(b2);
                return if2(new_pred, new_b1, new_b2);
            }
            Scheme::Begin (mut exprs) => {
                let tail = exprs.pop().unwrap();
                let new_tail = self.tail_helper(tail);
                let mut new_exprs: Vec<_> = exprs.into_iter().map(|x| self.effect_helper(x)).collect();
                new_exprs.push(new_tail);
                return Expr::Begin (new_exprs);
            }
            triv => self.scheme_to_expr(triv),
        }
    }
    
    fn pred_helper(&self, pred: Scheme) -> Expr {
        match pred {
            Scheme::Prim2 (relop, box v1, box v2) => {
                let new_v1 = self.value_helper(v1);
                let new_v2 = self.value_helper(v2);
                return Expr::Prim2 (relop, Box::new(new_v1), Box::new(new_v2));
            }
            Scheme::If (box pred, box b1, box b2) => {
                let new_pred = self.pred_helper(pred);
                let new_b1 = self.pred_helper(b1);
                let new_b2 = self.pred_helper(b2);
                return if2(new_pred, new_b1, new_b2);
            }
            Scheme::Begin (mut exprs) => {
                let pred = exprs.pop().unwrap();
                let new_pred = self.pred_helper(pred);
                let mut new_exprs: Vec<_> = exprs.into_iter().map(|x| self.effect_helper(x)).collect();
                new_exprs.push(new_pred);
                return Expr::Begin (new_exprs);
            }
            e => self.scheme_to_expr(e),
        }
    }
    

    fn effect_helper(&self, effect: Scheme) -> Expr {
        match effect {
            Scheme::Nop => Nop,
            Scheme::Mset (box v1, box v2, box v3) => {
                let new_v1 = self.value_helper(v1);
                let new_v2 = self.value_helper(v2);
                let new_v3 = self.value_helper(v3);
                return Expr::Mset (Box::new(new_v1), Box::new(new_v2), Box::new(new_v3));
            }
            Scheme::Funcall (box v, args) => {
                let new_v = self.value_helper(v);
                let new_args: Vec<_> = args.into_iter().map(|x| self.value_helper(x)).collect();
                return Expr::Funcall (Box::new(new_v), new_args);
            }
            Scheme::If (box pred, box b1, box b2) => {
                let new_pred = self.pred_helper(pred);
                let new_b1 = self.effect_helper(b1);
                let new_b2 = self.effect_helper(b2);
                return if2(new_pred, new_b1, new_b2); 
            }
            Scheme::Begin (exprs) => {
                let new_exprs: Vec<_> = exprs.into_iter().map(|x| self.effect_helper(x)).collect();
                return Expr::Begin (new_exprs);
            }
            Scheme::Set (box sym, box val) => {
                let new_sym = self.scheme_to_expr(sym);
                let new_val = self.value_helper(val);
                return set1(new_sym, new_val);
            }
            e => panic!("Invalid effect expression {}", e),
        }
    }

    fn value_helper(&self, value: Scheme) -> Expr {
        match value {
            Scheme::Prim2 (op, box v1, box v2) => {
                let new_v1 = self.value_helper(v1);
                let new_v2 = self.value_helper(v2);
                return Expr::Prim2 (op, Box::new(new_v1), Box::new(new_v2));
            }
            Scheme::Mref (box v1, box v2) => {
                let new_v1 = self.value_helper(v1);
                let new_v2 = self.value_helper(v2);
                return Expr::Mref (Box::new(new_v1), Box::new(new_v2));
            }
            Scheme::Alloc (box v) => Alloc (Box::new(self.value_helper(v))),
            Scheme::Funcall (box v, args) => {
                let new_v = self.value_helper(v);
                let new_args: Vec<_> = args.into_iter().map(|x| self.value_helper(x)).collect();
                return Expr::Funcall (Box::new(new_v), new_args);
            }
            Scheme::If (box pred, box b1, box b2) => {
                let new_pred = self.value_helper(pred);
                let new_b1 = self.value_helper(b1);
                let new_b2 = self.value_helper(b2);
                return if2(new_pred, new_b1, new_b2);
            }
            Scheme::Begin (mut exprs) => {
                let value = exprs.pop().unwrap();
                let value = self.value_helper(value);
                let mut new_exprs: Vec<_> = exprs.into_iter().map(|x| self.effect_helper(x)).collect();
                new_exprs.push(value);
                return Expr::Begin (new_exprs);
            }
            triv => self.scheme_to_expr(triv),
        }
    }
    
    fn scheme_to_expr(&self, scm: Scheme) -> Expr {
        match scm {
            Scheme::Bool (b) => Expr::Bool (b), 
            Scheme::Int64 (i) => Expr::Int64 (i), 
            Scheme::Symbol (s) => Expr::Symbol (s),
            c => panic!("Unexpect complex scheme {}", c),
        }
    }
}

// ---------------------------------------------------------------------
//
// the Expr Intermediate Language (UIL)
//
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

const ALIGN_SHIFT: i64 = 3;
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
    // When running test, data race happens
    static mut counter :usize = 5000;
    let mut s = String::from(prefix);
    unsafe {
        s.push_str(&counter.to_string());
        counter += 1;
    }
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
    format!("rpnt${}_{}", name.replace(".", "").replace("$", ""), salt)
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

// I decide to add (value value*) in A8. Just before we dive into Scheme
fn make_funcall(labl: String, args: Vec<Expr>) -> Expr {
    Funcall(Box::new(Symbol (labl)), args)
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
            Funcall (box mut func, mut args) => {
                let mut exprs = vec![];
                func = self.reduce_value(func, locals, &mut exprs);
                args = args.into_iter().map(|e| self.reduce_value(e, locals, &mut exprs)).collect();
                let funcall = Funcall (Box::new(func), args);
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
            Set (box sym, box Funcall (box mut func, mut args)) => {
                let mut exprs = vec![];
                func = self.reduce_value(func, locals, &mut exprs);
                args = args.into_iter().map(|e| self.reduce_value(e, locals, &mut exprs)).collect();
                let new_set = set1(sym, Funcall (Box::new(func), args));
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
            Funcall (box mut func, mut args) => {
                let mut exprs = vec![];
                func = self.reduce_value(func, locals, &mut exprs);
                args = args.into_iter().map(|e| self.reduce_value(e, locals, &mut exprs)).collect();
                let funcall = Funcall (Box::new(func), args);
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
            Funcall (box mut func, mut args) => {
                func = self.reduce_value(func, locals, prelude);
                args = args.into_iter().map(|e| self.reduce_value(e, locals, prelude)).collect();
                let funcall = Funcall (Box::new(func), args);
                let new_uvar = gen_uvar();
                let assign = set1(Symbol (new_uvar.clone()), funcall);
                prelude.push(assign);
                locals.insert(new_uvar.clone());
                return Symbol (new_uvar);
            }
            Alloc (box e) => {
                let new_e = self.reduce_value(e, locals, prelude);
                let new_uvar = gen_uvar();
                let assign = set1(Symbol (new_uvar.clone()), Alloc (Box::new(new_e)));
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
                exprs = exprs.into_iter().map(|e| self.effect_helper(e)).collect();
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
                let jump = Funcall (Box::new(Symbol (get_rp(rp))), args);
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
                let jump = Funcall (Box::new(Symbol (get_rp(rp))), args);
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
                let jump = Funcall (Box::new(Symbol (get_rp(rp))), args);
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
            Funcall (box Symbol (labl), mut args) => {
                let rp_label = get_rp_nontail(&labl);
                let mut exprs = vec![];
                let mut fv_assign = vec![];
                let mut liveset = vec![
                    Symbol (FRAME_POINTER_REGISTER.to_string()),
                    Symbol (RETRUN_ADDRESS_REGISTER.to_string()),
                    Symbol (ALLOCATION_REGISTER.to_string()),
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
                let new_call = Funcall (Box::new(Symbol (labl)), liveset);
                exprs.push(new_call);
                ReturnPoint (rp_label, Box::new(Begin (exprs)))
            }
            Set (box sym, box Funcall (labl, args)) => {
                let mut exprs = vec![];
                exprs.push(self.effect_helper(Funcall (labl, args), new_frame));
                exprs.push(set1(sym, Symbol (RETURN_VALUE_REGISTER.to_string())));
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
            Funcall (box Symbol (labl), args) => {
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
            Set (box Symbol(s), box Alloc (box e)) => {
                liveset.remove(s);
                self.record_conflicts(s, "", &liveset, conflict_graph);
                if let Symbol(s) = e { if is_uvar(s) || self.type_verify(s) {
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
            Prim2 (relop, box Int64 (i1), box Int64 (i2)) => self.relop_int_rewrite(relop, Int64 (i1), Int64 (i2), unspills),
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
                return self.set2_int_rewrite(a, op, Int64 (i1), Int64 (i2), unspills);
            }
            Set (box Symbol (a), box Symbol (b)) => {
                return self.set1_fv_rewrite(a, b, unspills);
            }
            Set (box Symbol (a), box Mref (box Int64 (base), box Int64 (offset))) => {
                return self.mref_int_rewrite(a, Int64 (base), Int64 (offset), unspills);
            }
            Set (box Symbol (a), box Mref (box base, box offset)) => {
                return self.mref_fv_rewrite(a, base, offset, unspills);
            }
            Set (box Symbol (a), box Alloc (box size)) => {
                let exprs = vec![
                    set1(Symbol (a), Symbol (ALLOCATION_REGISTER.to_string())),
                    make_alloc(size),
                ];
                return Begin (exprs);
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

    fn relop_int_rewrite(&self, relop: String, a: Expr, b: Expr, unspills: &mut HashSet<String>) -> Expr {
        let new_uvar = gen_uvar();
        unspills.insert(new_uvar.clone());
        let expr1 = set1(Symbol (new_uvar.clone()), a);
        let expr2 = Prim2 (relop, Box::new(Symbol (new_uvar)), Box::new(b));
        return Begin (vec![expr1, expr2]);
    }

    fn set1_fv_rewrite(&self, a: String, b: String, unspills: &mut HashSet<String>) -> Expr {
        if is_fv(&a) && (is_fv(&b) || is_label(&b)) {
            let new_uvar = gen_uvar();
            unspills.insert(new_uvar.clone());
            let expr1 = set1(Symbol (new_uvar.clone()), Symbol (b));
            let expr2 = set1(Symbol (a), Symbol (new_uvar));
            return Begin (vec![expr1, expr2]);
        }
        return set1(Symbol (a), Symbol (b));
    }

    fn set2_fv_rewrite(&self, a: String, op: String, b: String, c: String, unspills: &mut HashSet<String>) -> Expr {
        if (is_fv(&a) && is_fv(&c)) || (is_fv(&a) && op.as_str() == "*") {
            let new_uvar = gen_uvar();
            unspills.insert(new_uvar.clone());
            let expr1 = set1(Symbol (new_uvar.clone()), Symbol (c));
            let expr2 = set2(Symbol (a), op, Symbol (b), Symbol (new_uvar));
            return Begin (vec![expr1, expr2]);
        } 
        return set2(Symbol (a), op, Symbol (b), Symbol (c));
    }

    fn set2_int_rewrite(&self, a: String, op: String, b: Expr, c: Expr, unspills: &mut HashSet<String>) -> Expr {
        if is_fv(&a) && op.as_str() == "*" {
            let new_uvar = gen_uvar();
            unspills.insert(new_uvar.clone());
            let expr1 = set1(Symbol (new_uvar.clone()), b);
            let expr2 = set2(Symbol (new_uvar.clone()), op, Symbol (new_uvar.clone()), c);
            let expr3 = set1(Symbol (a), Symbol (new_uvar.clone()));
            return Begin (vec![expr1, expr2, expr3]);
        }
        let expr1 = set1(Symbol (a.clone()), b); 
        let expr2 = set2(Symbol (a.clone()), op, Symbol (a), c);
        return Begin (vec![expr1, expr2]);
    }
    
    fn replace_fv_label(&self, expr: Expr, unspills: &mut HashSet<String>, prelude: &mut Vec<Expr>) -> Expr {
        if let Symbol (s) = expr { 
            if is_fv(&s) || is_label(&s) {
                let new_uvar = gen_uvar();
                unspills.insert(new_uvar.clone());  
                prelude.push(set1(Symbol (new_uvar.clone()), Symbol (s)));
                return Symbol (new_uvar);
            } else { return Symbol (s); } 
        }
        return expr;
    }
    
    fn mref_int_rewrite(&self, mut a: String, base: Expr, offset: Expr, unspills: &mut HashSet<String>) -> Expr {
        if is_reg(&a) {
            let exprs = vec![
                set1(Symbol (a.clone()), base),
                set1(Symbol (a.clone()), Mref (Box::new(Symbol (a)), Box::new(offset))),
            ];
            return Begin (exprs);
        }
        let new_uvar = gen_uvar();
        unspills.insert(new_uvar.clone());
        let exprs = vec![
            set1(Symbol (new_uvar.clone()), base),
            set1(Symbol (new_uvar.clone()), Mref (Box::new(Symbol (new_uvar.clone())), Box::new(offset))),
            set1(Symbol (a), Symbol (new_uvar)),
        ];
        return Begin (exprs);
    }

    fn mref_fv_rewrite(&self, mut a: String, base: Expr, offset: Expr, unspills: &mut HashSet<String>) -> Expr {
        // so, base and offset should not be fv.
        // make sure a is a register
        let mut old_a = String::new();
        if !is_reg(&a) {
            old_a = a;
            a = gen_uvar();
            unspills.insert(a.clone());
        }
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
        let new_mref = set1(Symbol (a.clone()), Mref (Box::new(new_base), Box::new(new_offset)));
        exprs.push(new_mref);
        // restore old a
        if old_a.as_str() != "" {
            exprs.push( set1(Symbol (old_a), Symbol (a)) ) ;
        }
        return Begin (exprs);
    }

    fn mset_int_rewrite(&self, base: Expr, offset: Expr, value: Expr, unspills: &mut HashSet<String>) -> Expr {
        let mut exprs = vec![];
        let new_uvar = gen_uvar();
        unspills.insert(new_uvar.clone());  
        exprs.push(set1(Symbol (new_uvar.clone()), base)); 
        let new_value = self.replace_fv_label(value, unspills, &mut exprs);
        let new_mset = Mset (Box::new(Symbol (new_uvar)), Box::new(offset), Box::new(new_value));
        exprs.push(new_mset); 
        return Begin (exprs);
    }

    fn mset_fv_rewrite(&self, base: Expr, offset: Expr, value: Expr, unspills: &mut HashSet<String>) -> Expr {
        let mut exprs = vec![];
        let new_base = self.replace_fv_label(base, unspills, &mut exprs);
        let new_offset = self.replace_fv_label(offset, unspills, &mut exprs);
        let new_value = self.replace_fv_label(value, unspills, &mut exprs);
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
            Mref (box base, box offset) => {
                let new_base = self.finalize_frame_locations(bindings, base);
                let new_offset = self.finalize_frame_locations(bindings, offset);
                return Mref (Box::new(new_base), Box::new(new_offset));
            }
            Mset (box base, box offset, box value) => {
                let new_base = self.finalize_frame_locations(bindings, base);
                let new_offset = self.finalize_frame_locations(bindings, offset);
                let new_value = self.finalize_frame_locations(bindings, value);
                return Mset (Box::new(new_base), Box::new(new_offset), Box::new(new_value));
            }
            Prim2 (op, box e1, box e2) => {
                let new_e1 = self.finalize_frame_locations(bindings, e1);
                let new_e2 = self.finalize_frame_locations(bindings, e2);
                return Prim2 (op, Box::new(new_e1), Box::new(new_e2));
            },
            Funcall (box Symbol (name), mut args) => {
                args = args.into_iter().map(|e| self.finalize_frame_locations(bindings, e)).collect();
                match bindings.get(&name) {
                    None => Funcall (Box::new(Symbol (name)), args),
                    Some (loc) => Funcall (Box::new(Symbol (loc.to_string())), args),
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
            Funcall (box Symbol (name), args) => {
                match bindings.get(&name) {
                    None => Funcall (Box::new(Symbol (name)), args),
                    Some (loc) => Funcall (Box::new(Symbol (loc.to_string())), args),
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
            Funcall (box Symbol (mut labl), args) => {
                if is_fv(&labl) { labl = self.update_location(&labl, offset); }     
                return (Funcall (Box::new(Symbol (labl)), args), offset);
            }
            any => panic!("Invalid tail {}", any)
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
                let lab1 = gen_label();
                let new_b1 = self.tail_helper(b1, new_lambdas); 
                self.add_binding(&lab1, new_b1, new_lambdas);

                let lab2 = gen_label();
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
            Bool (true) => make_funcall(lab1.to_string(), vec![]),
            Bool (false) => make_funcall(lab2.to_string(), vec![]),
            If (box pred, box br1, box br2) => {
                let new_lab1 = gen_label();
                let new_br1 = self.pred_helper(br1, lab1, lab2, new_lambdas);
                self.add_binding(&new_lab1, new_br1, new_lambdas);

                let new_lab2 = gen_label();
                let new_br2 = self.pred_helper(br2, lab1, lab2, new_lambdas);
                self.add_binding(&new_lab2, new_br2, new_lambdas);
                
                return self.pred_helper(pred, &new_lab1, &new_lab2, new_lambdas);
            }
            relop => if2(relop, make_funcall(lab1.to_string(), vec![]), make_funcall(lab2.to_string(), vec![])),
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
                let lab_tail = gen_label();
                self.add_binding(&lab_tail, tail, new_lambdas);
                // first branch, jump to the join block
                let lab1 = gen_label();
                let new_b1 = self.effect_helper(b1, make_funcall(lab_tail.clone(), vec![]), new_lambdas);
                self.add_binding(&lab1, new_b1, new_lambdas);
                // second branch, jump to the join block too
                let lab2 = gen_label();
                let new_b2 = self.effect_helper(b2, make_funcall(lab_tail, vec![]), new_lambdas);
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
                If (relop, box Funcall (box Symbol (lab1), _), lab2) if &lab1 == next_lab => {
                    let not_relop = Prim1 ("not".to_string(), relop);
                    return If1 (Box::new(not_relop), lab2);
                }
                If (relop, lab1, box Funcall (box Symbol (lab2), _)) if &lab2 == next_lab => {
                    return If1 (relop, lab1);
                }
                Funcall (box Symbol (lab), _) if &lab == next_lab => {
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
                    self.op2("movq", RDI, self.string_to_reg(FRAME_POINTER_REGISTER)),
                    self.op2("movq", RSI, self.string_to_reg(ALLOCATION_REGISTER)),
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
            Set (box dst, box Mref (box Int64 (i), box reg)) | Set (box dst, box Mref (box reg, box Int64 (i))) => {
                let dst = self.expr_to_asm_helper(dst);
                let src = Deref (Box::new(self.expr_to_asm_helper(reg)), i);
                return self.op2("movq", src, dst);
            }
            Set (box dst, box Mref (box reg1, box reg2)) => {
                let dst = self.expr_to_asm_helper(dst);
                let src = DerefRegister (Box::new(self.expr_to_asm_helper(reg1)), Box::new(self.expr_to_asm_helper(reg2)));
                return self.op2("movq", src, dst);
            }
            Set (box dst, box src) => {
                let dst = self.expr_to_asm_helper(dst);
                let src = self.expr_to_asm_helper(src);
                return self.op2("movq", src, dst);
            },
            Mset (box Int64 (i), box reg, box value) | Mset (box reg, box Int64 (i), box value) => {
                let dst = Deref (Box::new(self.expr_to_asm_helper(reg)), i);
                let src = self.expr_to_asm_helper(value);
                match &src {
                    Label (s) => self.op2("leaq", DerefLabel (Box::new(RIP), Box::new(src)), dst),
                    other => self.op2("movq", src, dst),
                }
            }
            Mset (box reg1, box reg2, box value) => {
                let dst = DerefRegister (Box::new(self.expr_to_asm_helper(reg1)), Box::new(self.expr_to_asm_helper(reg2)));
                let src = self.expr_to_asm_helper(value);
                match &src {
                    Label (s) => self.op2("leaq", DerefLabel (Box::new(RIP), Box::new(src)), dst),
                    other => self.op2("movq", src, dst),
                }
            }
            Funcall (box Symbol (s), _) if is_fv(&s) => {
                let deref = self.fv_to_deref(&s);
                return Jmp (Box::new(deref));
            },
            Funcall (box Symbol (s), _) if is_reg(&s) => {
                let reg = self.string_to_reg(&s);
                return Jmp (Box::new(reg));
            },
            Funcall (box Symbol (s), _) => {
                let label = Label (s);
                return Jmp (Box::new(label));
            }
            If1 (box Prim1(op, box Prim2(relop, box v1, box v2)), box Funcall (box Symbol (s), _)) if op.as_str() == "not" => {
                let v1 = self.expr_to_asm_helper(v1);
                let v2 = self.expr_to_asm_helper(v2);
                let cond = self.op2("cmpq", v2, v1);
                let jmp = Jmpif (self.relop_to_cc(&relop, true).to_string(), Box::new(Label (s)));
                return Code (vec![cond, jmp]);
            }
            If1 (box Prim2(relop, box v1, box v2), box Funcall (box Symbol (s), _)) => {
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
    let expr = ParseScheme{}.run(s);
    compile_formatter("ParseScheme", &expr);
    let expr = OptimizeDirectCall{}.run(expr);
    compile_formatter("OptimizeDirectCall", &expr);
    let expr = RemoveAnonymousLambda{}.run(expr);
    compile_formatter("RemoveAnonymousLambda", &expr);
    let expr = SanitizeBindingForms{}.run(expr);
    compile_formatter("SanitizeBindingForms", &expr);
    let expr = UncoverFree{}.run(expr);
    compile_formatter("UncoverFree", &expr);
    let expr = ConvertClosure{}.run(expr);
    compile_formatter("ConvertClosure", &expr);
    let expr = OptimizeKnownCall{}.run(expr);
    compile_formatter("OptimizeKnownCall", &expr);
    let expr = IntroduceProceduraPrimitives{}.run(expr);
    compile_formatter("IntroduceProceduraPrimitives", &expr);
    let expr = LiftLetrec{}.run(expr);
    compile_formatter("LiftLetrec", &expr);
    let expr = NormalizeContext{}.run(expr);
    compile_formatter("NormalizeContext", &expr);
    let expr = SpecifyRepresentation{}.run(expr);
    compile_formatter("SpecifyRepresentation", &expr);
    let expr = UncoverLocals{}.run(expr);
    compile_formatter("UncoverLocals", &expr);
    let expr = RemoveLet{}.run(expr);
    compile_formatter("RemoveLet", &expr);
    let expr = CompileToExpr{}.run(expr);
    compile_formatter("CompileToExpr", &expr);
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
    let mut loop_id = 1;
    loop {
        println!("The {}-th iteration", loop_id);
        loop_id += 1;

        expr = FinalizeFrameLocations{}.run(expr);
        compile_formatter("FinalizeFrameLocations", &expr);
        expr = SelectInstructions{}.run(expr);
        compile_formatter("SelectInstructions", &expr);
        expr = UncoverRegisterConflict{}.run(expr);
        compile_formatter("UncoverRegisterConflict", &expr);
        expr = AssignRegister{}.run(expr);
        compile_formatter("AssignRegister", &expr);

        if everybody_home(&expr) {
            break;
        }

        expr = AssignFrame{}.run(expr);
        compile_formatter("AssignFrame", &expr);
    }
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
    let expr = CompileToAsm{}.run(expr);
    compile_formatter("CompileToAsm", &expr);
    return GenerateAsm{}.run(expr, filename)
}