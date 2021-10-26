use std::io::Write;
use std::fs::File;
use std::collections::HashMap;
use std::collections::HashSet;
use std::vec::IntoIter;
use uuid::Uuid;

use crate::syntax::{Expr, Asm, ConflictGraph};
use crate::parser::{Scanner, Parser};

use Expr::*;
use Asm::*;

// ---------------------- geenral predicate --------------------------------
const REGISTERS :[&str; 15] = ["rax", "rbx", "rcx", "rdx", "rsi", "rdi", "rbp",
                                "r8" , "r9" , "r10", "r11", "r12", "r13", "r14", "r15" ];
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

fn is_uvar(sym: &str) -> bool {
    let v: Vec<&str> = sym.split('.').collect();
    v.len() == 2 && v[0].len() > 0 && v[1].len() > 0
}

fn is_reg(reg: &str) -> bool {
    REGISTERS.contains(&reg)
}

fn is_fv(s: &str) -> bool {
    s.starts_with("fv")
}

fn is_label(sym: &str) -> bool {
    let v: Vec<&str> = sym.split('$').collect();
    v.len() == 2 && v[0].len() > 0 && v[1].len() > 0
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
    if let Begin (exprs) = expr {
        let mut new_exprs = vec![];
        helper(exprs, &mut new_exprs);
        return Begin (new_exprs); 
    }
    return expr;
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
            Locals (uvars, box tail) => Locals (uvars, Box::new(self.tail_helper(tail))),
            e => e,
        }
    }

    fn tail_helper(&self, tail: Expr) -> Expr {
        match tail {
            Funcall (labl, args) => {
                let mut collector = vec![];
                let new_args = args.into_iter().map(|x| self.simplify(x, &mut collector)).collect();
                let new_funcall = Funcall (labl, new_args);
                if collector.len() == 0 { 
                    return new_funcall;
                }
                collector.push( new_funcall );
                return flatten_begin(Begin (collector));
            },
            Prim2 (op, box e1, box e2) => {
                let mut collector = vec![];    
                let new_e1 = self.simplify(e1, &mut collector);
                let new_e2 = self.simplify(e2, &mut collector);
                let new_prim2 = Prim2 (op, Box::new(new_e1), Box::new(new_e2));
                if collector.len() == 0 {
                    return new_prim2;
                }
                collector.push(new_prim2);
                return flatten_begin(Begin (collector));
            },
            Begin (mut exprs) => {
                let tail = exprs.pop().unwrap();
                let new_tail = self.tail_helper(tail);
                exprs.push(new_tail);
                return flatten_begin(Begin (exprs));
            }
            If (box pred, box b1, box b2) => {
                let new_b1 = self.tail_helper(b1);
                let new_b2 = self.tail_helper(b2);
                let new_pred = self.pred_helper(pred);
                return If (Box::new(new_pred), Box::new(new_b1), Box::new(new_b2));
            }
            e => e,
        }
    }

    fn pred_helper(&self, pred: Expr) -> Expr {
        match pred {
            Prim2 (relop, box e1, box e2) => {
                let mut collector = vec![];
                let new_e1 = self.simplify(e1, &mut collector);
                let new_e2 = self.simplify(e2, &mut collector);
                let new_prim = Prim2 (relop, Box::new(new_e1), Box::new(new_e2));
                if collector.len() == 0 { 
                    return new_prim;
                }
                collector.push(new_prim);
                return flatten_begin(Begin (collector));
            }
            If (box pred, box br1, box br2) => {
                let new_br1 = self.pred_helper(br1);
                let new_br2 = self.pred_helper(br2);
                let new_pred = self.pred_helper(pred);
                return If (Box::new(new_pred), Box::new(new_br1), Box::new(new_br2));
            }
            Begin (mut exprs) => {
                let mut tail = exprs.pop().unwrap();
                tail = self.pred_helper(tail);
                exprs.push(tail);
                return flatten_begin(Begin (exprs));
            }
            boolean => boolean,
        }
    }

    fn simplify(&self, expr: Expr, collector: &mut Vec<Expr>) -> Expr {
        match expr {
            Prim2 (op, box e1, box e2) => {
                let new_e1 = self.simplify(e1, collector);
                let new_e2 = self.simplify(e2, collector);
                let new_prim = Prim2 (op, Box::new(new_e1), Box::new(new_e2));
                let tmp = gen_uvar();
                let set = Set (Box::new(Symbol (tmp.clone())), Box::new(new_prim));
                collector.push(set);
                return Symbol (tmp);
            }
            e => e,
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

pub trait UncoverConflict {
    fn type_verify(&self, s: &str) -> bool;
    fn uncover_conflict(&self, conflict_graph: ConflictGraph, tail: Expr) -> Expr;
    fn tail_liveset(&self, tail: &Expr, mut liveset: HashSet<String>, conflict_graph: &mut ConflictGraph) -> HashSet<String> {
        match tail {
            Funcall (labl, args) => {
                for a in args {
                    if let Symbol(s) = a { if self.type_verify(s) {
                        liveset.insert(s.to_string()); 
                    }}
                }
                if self.type_verify(labl) {
                    liveset.insert(labl.to_string());
                }
                return liveset;
            }
            If (box Bool(true), box b1, _) => self.tail_liveset(b1, liveset, conflict_graph),
            If (box Bool(false), _, box b2) => self.tail_liveset(b2, liveset, conflict_graph),
            If (box pred, box b1, box b2) => {
                let true_set = self.tail_liveset(b1, liveset.clone(), conflict_graph);
                let false_set = self.tail_liveset(b2, liveset, conflict_graph);
                return self.pred_liveset(pred, true_set, false_set, conflict_graph);
            }
            Begin (exprs) => {
                let exprs_slice = exprs.as_slice();
                let last = exprs.len() - 1;
                liveset = self.tail_liveset(&exprs_slice[last], liveset, conflict_graph);
                for i in (0..last).rev() {
                    liveset = self.effect_liveset(&exprs_slice[i], liveset, conflict_graph); 
                }
                return liveset;
            }
            e => panic!("Invalid Tail {}", tail),
        }   
    }

    fn pred_liveset(&self, pred: &Expr, true_liveset: HashSet<String>, fliveset: HashSet<String>, conflict_graph: &mut ConflictGraph) -> HashSet<String> {
        match pred {
            Bool (true) => true_liveset,
            Bool (false) => fliveset,
            If (box pred, box b1, box b2) => {
                let new_true_liveset = self.pred_liveset(b1, true_liveset.clone(), fliveset.clone(), conflict_graph);
                let new_fliveset = self.pred_liveset(b2, true_liveset, fliveset, conflict_graph);
                return self.pred_liveset(pred, new_true_liveset, new_fliveset, conflict_graph);
            }
            Begin (exprs) => {
                let exprs_slice = exprs.as_slice();
                let last = exprs.len() - 1;
                let mut liveset = self.pred_liveset(&exprs_slice[last], true_liveset, fliveset, conflict_graph);
                for i in 0..last {
                    liveset = self.effect_liveset(&exprs_slice[i], liveset, conflict_graph); 
                }
                return liveset;
            }
            Prim2 (relop, box v1, box v2 ) => {
                let mut liveset: HashSet<_> = self.liveset_union(true_liveset, fliveset);
                if let Symbol(s) = v1 { if is_uvar(s) {
                    liveset.insert(s.to_string());    
                }}
                if let Symbol(s) = v2 { if is_uvar(s) {
                    liveset.insert(s.to_string());
                }}
                return liveset;
            }
            e => panic!("Invalid Pred Expr {}", e),
        }
    }


    fn effect_liveset(&self, effect: &Expr, mut liveset: HashSet<String>, conflict_graph: &mut ConflictGraph) -> HashSet<String> {
        match effect {
            &Nop => liveset,
            If (box Bool(true), box b1, _) => self.effect_liveset(b1, liveset, conflict_graph),
            If (box Bool(false), _, box b2) => self.effect_liveset(b2, liveset, conflict_graph),
            If (box pred, box b1, box b2) => {
                let true_liveset = self.effect_liveset(b1, liveset.clone(), conflict_graph);
                let fliveset = self.effect_liveset(b2, liveset, conflict_graph);
                let liveset = self.liveset_union(true_liveset, fliveset);
                return self.effect_liveset(pred, liveset, conflict_graph);
            }
            Begin (exprs) => {
                for e in exprs {
                    liveset = self.effect_liveset(e, liveset, conflict_graph);
                }
                return liveset;
            }
            Set (box v1, box Prim2 (op, box v2, box v3)) => {
                if let Symbol(s) = v1 { 
                    liveset.remove(s);
                    self.record_conflicts(s, "", &liveset, conflict_graph);
                }
                if let Symbol(s) = v2 { if is_uvar(s) || self.type_verify(s) {
                    liveset.insert(s.to_string());
                }}
                if let Symbol(s) = v3 { if is_uvar(s) || self.type_verify(s) {
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
            Set (box v1, box v2) => {
                if let Symbol(s) = v1 { 
                    liveset.remove(s);
                    self.record_conflicts(s, "", &liveset, conflict_graph);
                }
                if let Symbol(s) = v2 { if is_uvar(s) || self.type_verify(s) {
                    liveset.insert(s.to_string());
                }}
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
            Locals (uvars, box tail) => {
                let mut conflict_graph = ConflictGraph::new();
                for uvar in uvars.iter() {
                    conflict_graph.insert(uvar.to_string(), HashSet::new());
                }
                let new_tail = self.uncover_conflict(conflict_graph, tail);
                return Locals (uvars, Box::new(new_tail));
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
        let _liveset = self.tail_liveset(&tail, HashSet::new(), &mut conflict_graph);
        return FrameConflict (conflict_graph, Box::new(tail));
    }
}

pub struct IntroduceAllocationForm {}
impl IntroduceAllocationForm {
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
            Locals (uvars, box tail) => {
                let new_tail = Ulocals (HashSet::new(), Box::new(Locate (HashMap::new(), Box::new(tail))));
                return Locals (uvars, Box::new(new_tail));
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
                    ">" => self.prim2("<", Symbol (sym), Int64 (i)),
                    ">=" => self.prim2("<=", Symbol (sym), Int64 (i)),
                    "<" => self.prim2(">", Symbol (sym), Int64 (i)),
                    "<=" => self.prim2(">=", Symbol (sym), Int64 (i)),
                    "=" => self.prim2("=", Symbol (sym), Int64 (i)),
                    op => panic!("Invalid relop {}", op),
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
                    return self.set2(Symbol (a), op, Symbol (b), Int64 (i));
                }
                return self.rewrite(a, op, Int64 (i), Symbol (b), unspills);
            }
            Set (box Symbol (a), box Prim2 (op, box Symbol (b), box Int64 (i))) => {
                if a == b {
                    return self.set2(Symbol (a), op, Symbol (b), Int64 (i));
                }
                return self.rewrite(a, op, Symbol (b), Int64 (i), unspills);
            }
            Set (box Symbol (a), box Symbol (b)) => {
                return self.set1_fv_rewrite(a, b, unspills);
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
            e => e,
        }
    }

    fn prim2(&self, op: &str, v1: Expr, v2: Expr) -> Expr {
        Prim2 (op.to_string(), Box::new(v1), Box::new(v2))
    }

    fn set2(&self, dst: Expr, op: String, opv1: Expr, opv2: Expr) -> Expr {
        let prim = Prim2 (op, Box::new(opv1), Box::new(opv2));
        Set (Box::new(dst), Box::new(prim))
    }

    fn set1(&self, dst: Expr, src: Expr) -> Expr {
        Set (Box::new(dst), Box::new(src))
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
            let expr1 = self.set1(Symbol (new_uvar.clone()), Symbol (b));
            let expr2 = Prim2 (relop, Box::new(Symbol (a)), Box::new(Symbol (new_uvar)));
            return Begin (vec![expr1, expr2]);
        }
        return Prim2 (relop, Box::new(Symbol (a)), Box::new(Symbol (b)));
    }

    fn set1_fv_rewrite(&self, a: String, b: String, unspills: &mut HashSet<String>) -> Expr {
        if is_fv(&a) && is_fv(&b) {
            let new_uvar = gen_uvar();
            unspills.insert(new_uvar.clone());
            let expr1 = self.set1(Symbol (new_uvar.clone()), Symbol (b));
            let expr2 = self.set1(Symbol (a), Symbol (new_uvar));
            return Begin (vec![expr1, expr2]);
        }
        return self.set1(Symbol (a), Symbol (b));
    }

    fn set2_fv_rewrite(&self, a: String, op: String, b: String, c: String, unspills: &mut HashSet<String>) -> Expr {
        if is_fv(&a) && is_fv(&c) {
            let new_uvar = gen_uvar();
            unspills.insert(new_uvar.clone());
            let expr1 = self.set1(Symbol (new_uvar.clone()), Symbol (c));
            let expr2 = self.set2(Symbol (a), op, Symbol (b), Symbol (new_uvar));
            return Begin (vec![expr1, expr2]);
        } 
        return self.set2(Symbol (a), op, Symbol (b), Symbol (c));
    }
    
    fn rewrite(&self, a: String, op: String, b: Expr, c: Expr, unspills: &mut HashSet<String>) -> Expr {
        let new_uvar = gen_uvar();
        unspills.insert(new_uvar.clone());
        let expr1 = self.set1(Symbol (new_uvar.clone()), b);
        let expr2 = self.set2(Symbol (new_uvar.clone()), op, Symbol (new_uvar.clone()), c);
        let expr3 = self.set1(Symbol (a), Symbol (new_uvar));
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
        let _liveset = self.tail_liveset(&tail, HashSet::new(), &mut conflict_graph);
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
        println!("variables {}", v);
        println!("conflict_graph {:?}", conflict_graph);
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
        let k = conflict_graph.len();

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
            Locate (bindings,  box tail)  =>  Locate (bindings, Box::new(self.discard_call_live(tail))),
            _ => unreachable!(),
        }
    }

    fn discard_call_live(&self, tail: Expr) -> Expr {
        match tail {
            Funcall (label, _args) => Funcall (label, vec![]),
            If (pred, box b1, box b2) => {
                let new_b1 = self.discard_call_live(b1);
                let new_b2 = self.discard_call_live(b2);
                return If (pred, Box::new(new_b1), Box::new(new_b2));
            }
            Begin (mut exprs) => {
                let new_tail = self.discard_call_live(exprs.pop().unwrap());
                exprs.push(new_tail);  
                return Begin (exprs);
            }
            e => panic!("Invalid tail {}", e),
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
                let new_b1 = self.effect_helper(b1, Funcall (lab_tail.clone(), vec![]), new_lambdas);
                self.add_binding(&lab1, new_b1, new_lambdas);
                // second branch, jump to the join block too
                let lab2 = self.gensym();
                let new_b2 = self.effect_helper(b2, Funcall (lab_tail, vec![]), new_lambdas);
                self.add_binding(&lab2, new_b2, new_lambdas);
                // since a single expr seq break into several blocks, an effect turn into a tail.
                return self.pred_helper(pred, &lab1, &lab2, new_lambdas);
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
                    self.op2("leaq", DerefLabel(Box::new(RIP), "_scheme_exit".to_string()), R15),
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
        let v :Vec<&str> = fv.split("fv").collect();
        let index :i64 = v[1].parse().unwrap();
        return Deref (Box::new(RBP), index * 8);
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
    Ok(())
    // let expr = UncoverFrameConflict{}.run(expr);
    // compile_formatter("UncoverFrameConflict", &expr);
    // let mut expr = IntroduceAllocationForm{}.run(expr);
    // compile_formatter("IntroduceAllocationForm", &expr);
    // let mut loop_id = 1;
    // loop {
    //     println!("The {}-th iteration", loop_id);
    //     loop_id += 1;
    //     expr = SelectInstructions{}.run(expr);
    //     compile_formatter("SelectInstructions", &expr);
    //     expr = UncoverRegisterConflict{}.run(expr);
    //     compile_formatter("UncoverRegisterConflict", &expr);
    //     expr = AssignRegister{}.run(expr);
    //     compile_formatter("AssignRegister", &expr);

    //     if everybody_home(&expr) {
    //         break;
    //     }

    //     expr = AssignFrame{}.run(expr);
    //     compile_formatter("AssignFrame", &expr);
    //     expr = FinalizeFrameLocations{}.run(expr);
    //     compile_formatter("FinalizeFrameLocations", &expr);
    // }
    // let expr = DiscardCallLive{}.run(expr);
    // compile_formatter("DiscardCallLive", &expr);
    // let expr = FinalizeLocations{}.run(expr);
    // compile_formatter("Finalizelocations", &expr);
    // let expr = ExposeBasicBlocks{}.run(expr);
    // compile_formatter("ExposeBasicBlocks", &expr);
    // let expr = OptimizeJump{}.run(expr);
    // compile_formatter("OptimizeJump", &expr);
    // let expr = FlattenProgram{}.run(expr);
    // compile_formatter("FlattenProgram", &expr);
    // let expr = CompileToAsm{}.run(expr);
    // compile_formatter("CompileToAsm", &expr);
    // return GenerateAsm{}.run(expr, filename)
}