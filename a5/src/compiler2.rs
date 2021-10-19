use std::io::Write;
use std::fs::File;
use std::collections::HashMap;
use std::collections::HashSet;
use std::vec::IntoIter;

use crate::syntax::{Expr, Asm, ConflictGraph};
use crate::parser::{Scanner, Parser};

use Expr::*;
use Asm::*;

// ---------------------- geenral predicate --------------------------------
const N_REG :usize = 15;
const registers :[&str; N_REG] = ["rax", "rbx", "rcx", "rdx", "rsi", "rdi", "rbp",
                                  "r8" , "r9" , "r10", "r11", "r12", "r13", "r14", "r15" ];

fn is_uvar(sym: &str) -> bool {
    let v: Vec<&str> = sym.split('.').collect();
    v.len() == 2 && v[0].len() > 0 && v[1].len() > 0
}

fn is_reg(reg: &str) -> bool {
    registers.contains(&reg)
}

fn is_fv(s: &str) -> bool {
    s.starts_with("fv")
}

fn is_label(sym: &str) -> bool {
    let v: Vec<&str> = sym.split('$').collect();
    v.len() == 2 && v[0].len() > 0 && v[1].len() > 0
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



pub trait UncoverConflict {
    fn type_verify(&self, s: &str) -> bool;
    fn uncover_conflict(&self, uvars: &Vec<Expr>, tail: Expr) -> Expr;
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
            Lambda (labl, box body) => Lambda (labl, Box::new(self.helper(body))),
            Locals (uvars, box tail) => {
                let new_tail = self.uncover_conflict(&uvars, tail);
                return Locals (uvars, Box::new(new_tail));
            }
            e => e,
        }
    }


    fn tail_liveset(&self, tail: &Expr, mut liveset: HashSet<String>, conflict_graph: &mut ConflictGraph) -> HashSet<String> {
        match tail {
            Funcall (labl, args) => {
                for a in args {
                    if let Symbol(s) = a { 
                        liveset.insert(s.to_string()); 
                    }
                }
                liveset.insert(labl.to_string());
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

pub struct UncoverRegisterConflict {}
impl UncoverConflict for UncoverRegisterConflict {
    fn type_verify(&self, s: &str) -> bool {
        is_reg(s)
    }

    fn uncover_conflict(&self, uvars: &Vec<Expr>, tail: Expr) -> Expr {
        let mut conflict_graph = ConflictGraph::new();
        for uvar in uvars {
            conflict_graph.insert(uvar.to_string(), HashSet::new());
        }
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
            Lambda (label, box body) => Lambda (label, Box::new(self.helper(body))),
            Locals (_, box RegisterConflict (conflict_graph, box tail) ) =>  self.assign_registers(conflict_graph, tail),
            _ => unreachable!(),
        }
    }
    fn assign_registers(&self, mut conflict_graph: ConflictGraph, tail: Expr) -> Expr {
        let mut assigned = HashMap::new();
        let mut available = registers.iter().map(|reg| reg.to_string()).collect();
        self.assign_helper(&mut conflict_graph, &mut assigned, &mut available);
        return Locate (assigned, Box::new(tail));
    }

    fn assign_helper(&self, conflict_graph: &mut ConflictGraph, assigned: &mut HashMap<String, String>, available: &mut HashSet<String>) {
        if conflict_graph.len() == 0 { return; }
        let v = self.proposal_var(conflict_graph);
        let conflicts = conflict_graph.remove(&v).unwrap();
        // remove the picked variable from the conflict graph
        for set in conflict_graph.values_mut() {
            set.remove(&v);
        }
        // assign other variable firstlu
        self.assign_helper(conflict_graph, assigned, available);
        // assign the picked variables
        let reg = self.find_available(conflicts, available);
        available.remove(&reg);
        assigned.insert(v, reg);
    }

    // find the low-degree variable
    fn proposal_var(&self, conflict_graph: &ConflictGraph) -> String {
        let mut v = "";
        let mut degree = usize::max_value();
        for (k, list) in conflict_graph {
            if list.len() < degree {
                v = k;
                degree = list.len();
            }
        }
        return v.to_string();
    }

    fn find_available(&self, conflict: HashSet<String>, available: &HashSet<String>) -> String {
        for reg in available.difference(&conflict) {
            return reg.to_string();
        }
        panic!("Unable to find a available register!");
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
            Lambda (label, box body) => Lambda (label, Box::new(self.helper(body))),
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


pub fn compile_formater<T: std::fmt::Display>(s: &str, expr: &T) {
    println!(">>> {}", s);
    println!("----------------------------");
    println!("{}", expr);
    println!("----------------------------\n");
}

pub fn compile(s: &str, filename: &str) -> std::io::Result<()>  {
    let expr = ParseExpr{}.run(s);
    compile_formater("ParseExpr", &expr);
    let expr = UncoverRegisterConflict{}.run(expr);
    compile_formater("UncoverRegisterCOnflict", &expr);
    let expr = AssignRegister{}.run(expr);
    compile_formater("AssignRegister", &expr);
    let expr = DiscardCallLive{}.run(expr);
    compile_formater("DiscardCallLive", &expr);
    let expr = FinalizeLocations{}.run(expr);
    compile_formater("Finalizelocations", &expr);
    let expr = ExposeBasicBlocks{}.run(expr);
    compile_formater("ExposeBasicBlocks", &expr);
    let expr = OptimizeJump{}.run(expr);
    compile_formater("OptimizeJump", &expr);
    let expr = FlattenProgram{}.run(expr);
    compile_formater("FlattenProgram", &expr);
    let expr = CompileToAsm{}.run(expr);
    compile_formater("CompileToAsm", &expr);
    return GenerateAsm{}.run(expr, filename)
}