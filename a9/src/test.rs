use std::process::Command;
use crate::compiler::compile;
use crate::syntax::Expr;


fn run_helper(filename: &str) -> String {
    let obj: Vec<&str> = filename.split(".").collect();
    let stem = format!("test_{}", &obj[0]);
    let output = Command::new("gcc")
                    .arg("-m64")
                    .arg("-o")
                    .arg(&stem)
                    .arg(filename)
                    .arg("runtime.c")
                    .output()
                    .expect("failed to execute process");
    println!("{}", String::from_utf8_lossy(&output.stderr));
    let output = Command::new(stem).output().expect("failed to execute process");
    return String::from_utf8_lossy(&output.stdout).to_string();
}

fn test_helper(program: &str, filename: &str, expect: i64) {
    compile(program, filename);
    let r = run_helper(filename);
    let expect_str = format!("{}\n", expect);
    assert_eq!(r, expect_str);
}

#[test]
fn compile1() {
    let s =     
    "(letrec ([f$0 (lambda (p.2) (- (mref p.2 8) (mref p.2 0)))])
      (let ([x.1 (alloc 16)])
        (begin
          (mset! x.1 0 73)
          (mset! x.1 8 35)
          (f$0 x.1))))";
    test_helper(s, "c1.s", -38);
}

#[test]
fn compile2() {
    let s = "
    (letrec ([f$0 (lambda (p.2 i.3 i.4) (- (mref p.2 i.3) (mref p.2 i.4)))])
      (let ([x.1 (alloc 16)])
        (begin
          (mset! x.1 0 73)
          (mset! x.1 8 35)
          (+ (f$0 x.1 0 8) -41))))";
    test_helper(s, "c2.s", -3);
}

#[test]
fn compile3() {
    let s = "
    (letrec ([make-vector$0 (lambda (size.1)
                              (let ([v.2 (alloc (+ (* size.1 8) 8))])
                                (begin
                                  (mset! 0 v.2 size.1)
                                  v.2)))]
             [chained-vector-set!$1 (lambda (v.3 off.4 val.5)
                                      (begin
                                        (mset! (* (+ off.4 1) 8) v.3 val.5)
                                        v.3))]
             [vector-length$4 (lambda (v.8) (mref v.8 0))]
             [find-greatest-less-than$2 (lambda (v.6 val.7)
                                          (fglt-help$3 v.6 val.7 (+ v.6 8)
                                            (vector-length$4 v.6)))]
             [fglt-help$3 (lambda (v.9 val.10 curr.11 size.12)
                            (if (if (> curr.11 (+ (+ v.9 (* size.12 8)) 8))
                                    (true)
                                    (> (mref curr.11 0) val.10))
                                (mref curr.11 -8)
                                (fglt-help$3 v.9 val.10 (+ curr.11 8)
                                             size.12)))])
      (let ([v.13 (chained-vector-set!$1
                    (chained-vector-set!$1 
                      (chained-vector-set!$1 
                        (chained-vector-set!$1 
                          (chained-vector-set!$1 
                            (chained-vector-set!$1 
                              (chained-vector-set!$1 
                                (chained-vector-set!$1 
                                  (chained-vector-set!$1 
                                    (chained-vector-set!$1 
                                      (make-vector$0 10) 0 0)
                                    1 10)
                                  2 20)
                                3 30)
                              4 40)
                            5 50)
                          6 60)
                        7 70)
                      8 80)
                    9 90)])
        (find-greatest-less-than$2 v.13 76)))";
    test_helper(s, "c3.s", 70);
}

#[test]
fn compile4() {
    let s = "
    (letrec ([vector-scale!$0 (lambda (vect.1 scale.2)
                                (let ([size.3 (mref vect.1 0)])
                                  (vector-scale!$1 size.3 vect.1 scale.2)))]
             [vector-scale!$1 (lambda (offset.4 vect.5 scale.6)
                                (if (< offset.4 1)
                                    0
                                    (begin
                                      (mset! vect.5 (* offset.4 8)
                                             (* (mref vect.5 (* offset.4 8))
                                                scale.6))
                                      (vector-scale!$1 (- offset.4 1)
                                                       vect.5 scale.6))))]
             [vector-sum$2 (lambda (vect.7)
                             (vector-sum$3 (mref vect.7 0) vect.7))]
             [vector-sum$3 (lambda (offset.9 vect.10)
                             (if (< offset.9 1)
                                 0
                                 (+ (mref vect.10 (* offset.9 8))
                                    (vector-sum$3 (- offset.9 1)
                                                  vect.10))))])
      (let ([vect.11 (alloc 48)])
        (begin
          (mset! vect.11 0 5)
          (mset! vect.11 8 123)
          (mset! vect.11 16 10)
          (mset! vect.11 24 7)
          (mset! vect.11 32 12)
          (mset! vect.11 40 57)
          (vector-scale!$0 vect.11 10)
          (vector-sum$2 vect.11))))";
    test_helper(s, "c4.s", 2090);
}

#[test]
fn compile5() {
    let s = "
    (letrec ([thunk-num$0 (lambda (n.1)
                            (let ([th.2 (alloc 16)])
                              (begin 
                                (mset! th.2 0 force-th$1)
                                (mset! th.2 8 n.1)
                                th.2)))]
             [force-th$1 (lambda (cl.3)
                           (mref cl.3 8))]
             [add-ths$2 (lambda (cl1.4 cl2.5 cl3.6 cl4.7)
                          (+ (+ ((mref cl1.4 0) cl1.4)
                                ((mref cl2.5 0) cl2.5))
                             (+ ((mref cl3.6 0) cl3.6)
                                ((mref cl4.7 0) cl4.7))))])
      (add-ths$2 (thunk-num$0 5) (thunk-num$0 17) (thunk-num$0 7)
                 (thunk-num$0 9)))";
    test_helper(s, "c5.s", 38);
}

#[test]
fn compile6() {
    let s = "
    (letrec ()
      (let ([n.1 5])
        (let ([a.2 1])
          (let ([a.3 (* a.2 n.1)])
            (let ([n.4 (- n.1 1)])
              (let ([a.5 (* a.3 n.4)])
                (let ([n.6 (- n.4 1)])
                  (let ([a.7 (* a.5 n.6)])
                    (let ([n.8 (- n.6 1)])
                      (let ([a.9 (* a.7 n.8)])
                        a.9))))))))))";
    test_helper(s, "c6.s", 120);
}

#[test]
fn compile7() {
    let s = "
    (letrec ()
       (let ([a.1 1] [b.2 2])
         (let ([c.3 a.1] [d.4 4] [e.5 5] [f.6 b.2])
           (let ([f.16 (+ f.6 c.3)])
             (let ([f.26 (+ f.16 d.4)])
               (let ([f.36 (+ f.26 e.5)] [g.7 7])
                 (+ f.36 g.7)))))))";
    test_helper(s, "c7.s", 19);
}

#[test]
fn compile8() {
    let s = "    
    (letrec ()
      (let ([a.1 1]
            [b.2 2]
            [c.3 3]
            [d.4 4]
            [e.5 5]
            [f.6 6]
            [g.7 7]
            [h.8 8]
            [i.9 9]
            [j.10 10]
            [k.11 11]
            [l.12 12]
            [m.13 13]
            [n.14 14]
            [o.15 15]
            [p.16 16]
            [q.17 17]
            [r.18 18]
            [s.19 19]
            [t.20 20]
            [u.21 21]
            [v.22 22]
            [w.23 23]
            [x.24 24]
            [y.25 25]
            [z.26 26])
        (let ([a.101 (+ a.1 (+ b.2 (+ c.3 (+ d.4 (+ e.5 (+ f.6 (+ g.7 (+ h.8
                     (+ i.9 (+ j.10 (+ k.11 (+ l.12 (+ m.13 (+ n.14 (+ o.15 
                     (+ p.16 (+ q.17 (+ r.18 (+ s.19 (+ t.20 (+ u.21 (+ v.22 
                     (+ w.23 (+ x.24 (+ y.25 z.26)))))))))))))))))))))))))]
              [b.202 27]
              [c.203 28]
              [d.204 29]
              [e.205 30]
              [f.206 31]
              [g.207 32]
              [h.208 33]
              [i.209 34]
              [j.2010 35]
              [k.2011 36]
              [l.2012 37]
              [m.2013 38]
              [n.2014 39]
              [o.2015 40])
          (let ([a.102 (+ a.101 (+ b.202 (+ c.203 (+ d.204 (+ e.205 (+ f.206 (+ g.207 (+ h.208
                       (+ i.209 (+ j.2010 (+ k.2011 (+ l.2012 (+ m.2013 
                       (+ n.2014 o.2015))))))))))))))])
            (+ a.102 a.1)))))))";
    test_helper(s, "c8.s", 821);
}