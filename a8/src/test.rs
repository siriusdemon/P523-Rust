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
    let s = "
    (letrec ()
      (locals (a.1 x.2)
        (begin
          (set! x.2 (alloc 16))
          (mset! x.2 8 3)
          (mref (begin (if (< 40 50)
                           (set! a.1 x.2)
                           (set! a.1 x.2))
                       a.1)
                (begin (set! a.1 8) a.1)))))";
    test_helper(s, "c1.s", 3);
}

#[test]
fn compile2() {
    let s = "
    (letrec ([member$0 (lambda (x.1 ls.2)
                         (locals (size.4 ls.3)
                           (begin
                             (set! size.4 (mref ls.2 0))
                             (if (> x.1 size.4)
                                 0
                                 (if (= x.1 (mref ls.2 16))
                                     1
                                     (begin
                                       (set! ls.3 (alloc (* 8 (- size.4 1))))
                                       (mset! ls.3 0 (- size.4 1))
                                       (mset! ls.3 8 (+ ls.2 16))
                                       (member$0 x.1 ls.3)))))))])
      (locals (ls.1)
        (begin
          (set! ls.1 (alloc 48))
          (mset! ls.1 0 5)
          (mset! ls.1 8 9)
          (mset! ls.1 16 2)
          (mset! ls.1 24 7)
          (mset! ls.1 32 8)
          (mset! ls.1 40 3)
          (member$0 4 ls.1))))";
    test_helper(s, "c2.s", 0);
}

#[test]
fn compile3() {
    let s = "
    (letrec ([a$1 (lambda (m.5 x.1 y.2)
                    (locals ()
                      (begin
                        (mset! m.5 0 (+ x.1 y.2))
                        m.5)))])
      (locals (x.3)
        (begin
          (set! x.3 (a$1 (alloc 8) 10 6))
          (mref x.3 0))))";
    test_helper(s, "c3.s", 16);
}

#[test]
fn compile4() {
    let s = "  
    (letrec ([a$1 (lambda (m.1 a.2)
                    (locals ()
                      (begin
                        (mset! m.1 a.2 (+ (mref m.1 (- a.2 8))
                                          (mref m.1 (- a.2 8))))
                        1)))])
      (locals (m.3)
        (begin
          (set! m.3 (alloc 56))
          (mset! m.3 0 1)
          (mset! m.3 8 1)
          (a$1 m.3 16)
          (a$1 m.3 24)
          (a$1 m.3 32)
          (a$1 m.3 40)
          (a$1 m.3 48)
          (mref m.3 48))))";
    test_helper(s, "c4.s", 32);
}


#[test]
fn compile5() {
    let s = "
    (letrec ([f$0 (lambda (c.3 d.4)
                    (locals (e.5)
                      (- (mref c.3 (mref d.4 8))
                         (begin
                           (set! e.5 (alloc 16))
                           (mset! e.5 0 (mref c.3 0))
                           (mset! e.5 8 (mref d.4 0))
                           (if (> (mref e.5 0) (mref e.5 8))
                               (mref e.5 8)
                               (mref e.5 0))))))])
      (locals (a.1 b.2)
        (begin
          (set! a.1 (alloc 24))
          (set! b.2 (alloc 16))
          (mset! a.1 0 8)
          (mset! a.1 8 (+ (mref a.1 0) (mref a.1 0)))
          (mset! a.1 16 (+ (mref a.1 0) (mref a.1 8)))
          (mset! b.2 0 (mref a.1 16))
          (mset! b.2 8 (- (mref b.2 0) (mref a.1 0)))
          (f$0 a.1 b.2))))";
    test_helper(s, "c5.s", 16);
}


#[test]
fn compile6() {
    let s = "
    (letrec ([f$1 (lambda (x.1) (locals () (mref x.1 0)))]
             [f$2 (lambda (x.2) (locals () (mref x.2 8)))])
      (locals (z.3)
        (begin
          (set! z.3 (alloc 32))
          (mset! z.3 0 5)
          (mset! z.3 8 12)
          (+ (f$1 z.3) (f$2 z.3)))))";
    test_helper(s, "c6.s", 17);
}

#[test]
fn compile7() {
    let s = "
    (letrec ([f$0 (lambda (a.2) (locals () (+ (mref a.2 0) 13)))])
      (locals (y.1)
        (begin
          (set! y.1 (alloc 16))
          (mset! y.1 0 10)
          (f$0 y.1))))";
    test_helper(s, "c7.s", 23);
}

#[test]
fn compile8() {
    let s = "
    (letrec ()
      (locals ()
        (if (if (= (+ 7 (* 2 4)) (- 20 (+ (+ 1 1) (+ (+ 1 1) 1))))
                (> 2 3)
                (< 15 (* 4 4)))
            (+ 1 (+ 2 (+ 3 (+ 4 5))))
            0)))";
    test_helper(s, "c8.s", 0);
}

#[test]
fn compile9() {
    let s = "
     (letrec ([vector-scale!$0 (lambda (vect.1 scale.2)
                                (locals (size.3)
                                  (begin
                                    (set! size.3 (mref vect.1 0))
                                    (vector-scale!$1 size.3 vect.1 scale.2))))]
             [vector-scale!$1 (lambda (offset.4 vect.5 scale.6)
                                (locals ()
                                  (if (< offset.4 1)
                                      0
                                      (begin
                                        (mset! vect.5 (* offset.4 8)
                                               (* (mref vect.5 (* offset.4 8))
                                                  scale.6))
                                        (vector-scale!$1 (- offset.4 1)
                                                         vect.5 scale.6)))))]
             [vector-sum$2 (lambda (vect.7)
                             (locals ()
                               (vector-sum$3 (mref vect.7 0) vect.7)))]
             [vector-sum$3 (lambda (offset.9 vect.10)
                             (locals ()
                               (if (< offset.9 1)
                                   0
                                   (+ (mref vect.10 (* offset.9 8))
                                      (vector-sum$3 (- offset.9 1)
                                                    vect.10)))))])
      (locals (vect.11)
        (begin
          (set! vect.11 (alloc 48))
          (mset! vect.11 0 5)
          (mset! vect.11 8 123)
          (mset! vect.11 16 10)
          (mset! vect.11 24 7)
          (mset! vect.11 32 12)
          (mset! vect.11 40 57)
          (vector-scale!$0 vect.11 10)
          (vector-sum$2 vect.11))))";
    test_helper(s, "c9.s", 2090);
}


#[test]
fn compile10() {
    let s = "
    (letrec ([cc$0 (lambda (fst.1 snd.2)
                     (locals (ptr.3)
                       (begin
                         (set! ptr.3 (alloc 16))
                         (mset! ptr.3 0 fst.1)
                         (mset! ptr.3 8 snd.2)
                         ptr.3)))]
             [fst$1 (lambda (ptr.4) (locals () (mref ptr.4 0)))]
             [snd$2 (lambda (ptr.5) (locals () (mref ptr.5 8)))]
             [length$3 (lambda (ptr.6)
                         (locals ()
                           (if (= ptr.6 0)
                               0
                               (+ 1 (length$3 (snd$2 ptr.6))))))])
      (locals ()
        (length$3 (cc$0 5 (cc$0 10 (cc$0 11 (cc$0 5 (cc$0 15 0))))))))";
    test_helper(s, "c10.s", 5);
}

#[test]
fn compile11() {
    let s = "
    (letrec ([cc$0 (lambda (fst.1 snd.2)
                     (locals (ptr.3)
                       (begin
                         (set! ptr.3 (alloc 16))
                         (mset! ptr.3 0 fst.1)
                         (mset! ptr.3 8 snd.2)
                         ptr.3)))]
             [fst$1 (lambda (ptr.4) (locals () (mref ptr.4 0)))]
             [snd$2 (lambda (ptr.5) (locals () (mref ptr.5 8)))]
             [count-leaves$3 (lambda (ptr.6)
                               (locals ()
                                 (if (= ptr.6 0)
                                     1
                                     (+ (count-leaves$3 (fst$1 ptr.6))
                                        (count-leaves$3 (snd$2 ptr.6))))))])
      (locals ()
        (count-leaves$3
          (cc$0 
            (cc$0
              0
              (cc$0 0 0))
            (cc$0
              (cc$0
                (cc$0 (cc$0 0 (cc$0 0 0)) 0)
                0)
              (cc$0 (cc$0 (cc$0 0 0) (cc$0 0 (cc$0 0 0)))
                    (cc$0 (cc$0 0 0) 0)))))))";
    test_helper(s, "c11.s", 16);
}


#[test]
fn compile12() {
    let s = "
    (letrec ([cc$0 (lambda (fst.1 snd.2)
                     (locals (ptr.3)
                       (begin
                         (set! ptr.3 (alloc 16))
                         (mset! ptr.3 0 fst.1)
                         (mset! ptr.3 8 snd.2)
                         ptr.3)))]
             [fst$1 (lambda (ptr.4) (locals () (mref ptr.4 0)))]
             [snd$2 (lambda (ptr.5) (locals () (mref ptr.5 8)))]
             [add1$3 (lambda (n.6) (locals () (+ n.6 1)))]
             [map$4 (lambda (f.7 ls.8)
                      (locals ()
                        (if (= ls.8 0)
                            0
                            (cc$0 (f.7 (fst$1 ls.8)) 
                                  (map$4 f.7 (snd$2 ls.8))))))]
             [sum$5 (lambda (ls.9)
                      (locals ()
                        (if (= 0 ls.9)
                            0
                            (+ (fst$1 ls.9) (sum$5 (snd$2 ls.9))))))])
      (locals (ls.10)
        (begin
          (set! ls.10 (cc$0 5 (cc$0 4 (cc$0 3 (cc$0 2 (cc$0 1 0))))))
          (set! ls.10 (cc$0 10 (cc$0 9 (cc$0 8 (cc$0 7 (cc$0 6 ls.10))))))
          (sum$5 (map$4 add1$3 ls.10)))))";
    test_helper(s, "c12.s", 65);
}


#[test]
fn compile_high_order_fn() {
    let s = "
    (letrec ([high_order$0 (lambda (f.1 a.2)
                            (locals ()
                                (f.1 a.2)))]
             [add$1 (lambda (b.3) (locals () (+ b.3 1)))])
      (locals ()
        (high_order$0 add$1 10)))";
    
    test_helper(s, "high.s", 11);
}

#[test]
fn compile122() {
    let s = "
    (letrec ([cc$0 (lambda (fst.1 snd.2)
                     (locals (ptr.3)
                       (begin
                         (set! ptr.3 (alloc 16))
                         (mset! ptr.3 0 fst.1)
                         (mset! ptr.3 8 snd.2)
                         ptr.3)))]
             [fst$1 (lambda (ptr.4) (locals () (mref ptr.4 0)))]
             [snd$2 (lambda (ptr.5) (locals () (mref ptr.5 8)))]
             [add1$3 (lambda (n.6) (locals () (+ n.6 1)))]
             [map$4 (lambda (f.7 ls.8)
                      (locals ()
                        (if (= ls.8 0)
                            0
                            (cc$0 (f.7 (fst$1 ls.8)) 
                                  (map$4 f.7 (snd$2 ls.8))))))]
             [sum$5 (lambda (ls.9)
                      (locals ()
                        (if (= 0 ls.9)
                            0
                            (+ (fst$1 ls.9) (sum$5 (snd$2 ls.9))))))])
      (locals (ls.10)
        (begin
          (set! ls.10 (cc$0 5 (cc$0 4 (cc$0 3 (cc$0 2 (cc$0 1 0))))))
          (set! ls.10 (cc$0 10 (cc$0 9 (cc$0 8 (cc$0 7 (cc$0 6 ls.10))))))
          (sum$5 ls.10))))";
    test_helper(s, "c122.s", 55);
}


#[test]
fn compile13() {
    let s = "
    (letrec ([proc$1 (lambda (a.1)
                       (locals ()
                         (begin
                           (+ a.1 5))))])
      (locals (b.1)
        (begin
          (set! b.1 (alloc 8))
          (mset! b.1 0 proc$1)
          (set! b.1 (mref b.1 0))
          (b.1 4))))";
    test_helper(s, "c13.s", 9);
}


#[test]
fn compile14() {
    let s = "
    (letrec ()
      (locals (x.15)
        (begin
          (set! x.15 (alloc 16))
          (if (= 10 11) (nop) (mset! x.15 0 12))
          (set! x.15 x.15)
          (mset! x.15 8 x.15)
          (set! x.15 (mref x.15 8))
          (mref x.15 0))))";
    test_helper(s, "c14.s", 12);
}

#[test]
fn compile15() {
    let s = "
    (letrec ([make-list$0 (lambda (length.0)
                            (locals ()
                              (alloc (byte-offset$3 length.0))))]
             [fill-list$1 (lambda (content.0 value.1 length.2)
                            (locals ()
                              (fill-list-helper$2 content.0
                                                  value.1
                                                  0
                                                  length.2)))]
             [fill-list-helper$2 (lambda (content.0 value.1 index.2 length.3)
                                   (locals ()
                                     (if (= index.2 length.3)
                                         content.0
                                         (begin
                                           (mset! content.0
                                                  (byte-offset$3 index.2)
                                                  value.1)
                                           (fill-list-helper$2 content.0
                                                               value.1
                                                               (+ index.2 1)
                                                               length.3)))))]
             [byte-offset$3 (lambda (int.0)
                              (locals ()
                                (* int.0 8)))])
      (locals (length.10 value.5)
        (begin
          (set! length.10 10)
          (set! value.5 5)
          (mref (fill-list$1 (make-list$0 length.10)
                             value.5
                             length.10)
                (byte-offset$3 (- length.10 1))))))";
    test_helper(s, "c15.s", 5);
}

// func still not a value
#[test]
#[should_panic()]
fn compile16() {
    let s = "
    (letrec ([thunk-num$0 (lambda (n.1)
                            (locals (th.2)
                              (begin 
                                (set! th.2 (alloc 16))
                                (mset! th.2 0 force-th$1)
                                (mset! th.2 8 n.1)
                                th.2)))]
             [force-th$1 (lambda (cl.3)
                           (locals ()
                             (mref cl.3 8)))]
             [add-ths$2 (lambda (cl1.4 cl2.5 cl3.6 cl4.7)
                          (locals ()
                            (+ (+ ((mref cl1.4 0) cl1.4)
                                  ((mref cl2.5 0) cl2.5))
                               (+ ((mref cl3.6 0) cl3.6)
                                  ((mref cl4.7 0) cl4.7)))))])
      (locals ()
        (add-ths$2 (thunk-num$0 5) (thunk-num$0 17) (thunk-num$0 7)
                   (thunk-num$0 9))))";
    test_helper(s, "c16.s", 38);
}

#[test]
fn compile17() {
    let s = "
    (letrec ([f$0 (lambda (p.2 i.3 i.4)
                    (locals () (- (mref p.2 i.3) (mref p.2 i.4))))])
      (locals (x.1)
        (begin
          (set! x.1 (alloc 16))
          (mset! x.1 0 73)
          (mset! x.1 8 35)
          (+ (f$0 x.1 0 8) -41))))";
    test_helper(s, "c17.s", -3);
}

#[test]
fn compile18() {
    let s = "
    (letrec ([f$0 (lambda (p.3)
                    (locals (p.4)
                      (- (mref
                           (mref (mref (mref (mref p.3 0) 0) 8) 0)
                           (mref (mref p.3 8) (mref (mref p.3 0) 32)))
                         (mref
                           (mref p.3 (mref p.3 16))
                           (mref (mref p.3 0) (mref p.3 32))))))])
      (locals (x.1 x.2)
        (begin
          (set! x.1 (alloc 48))
          (set! x.2 (alloc 56))
          (mset! x.1 0 x.2)
          (mset! x.1 8 x.1)
          (mset! x.2 0 x.1)
          (mset! x.2 8 -4421)
          (mset! x.1 16 0)
          (mset! x.1 24 -37131)
          (mset! x.1 32 32)
          (mset! x.1 40 48)
          (mset! x.2 16 -55151)
          (mset! x.2 24 -32000911)
          (mset! x.2 32 40)
          (mset! x.2 40 55)
          (mset! x.2 48 -36)
          (* (f$0 x.1) 2))))";
    test_helper(s, "c18.s", -182);
}

#[test]
fn compile19() {
    let s = "
    (letrec ([make-vector$0 (lambda (size.1)
                              (locals (v.2)
                                (begin
                                  (set! v.2 (alloc (+ (* size.1 8) 8)))
                                  (mset! 0 v.2 size.1)
                                  v.2)))]
             [chained-vector-set!$1 (lambda (v.3 off.4 val.5)
                                      (locals ()
                                        (begin
                                          (mset! (* (+ off.4 1) 8) v.3 val.5)
                                          v.3)))]
             [vector-length$4 (lambda (v.8) (locals () (mref v.8 0)))]
             [find-greatest-less-than$2 (lambda (v.6 val.7)
                                          (locals ()
                                            (fglt-help$3 v.6 val.7 (+ v.6 8)
                                              (vector-length$4 v.6))))]
             [fglt-help$3 (lambda (v.9 val.10 curr.11 size.12)
                            (locals ()
                              (if (if (> curr.11 (+ (+ v.9 (* size.12 8)) 8))
                                      (true)
                                      (> (mref curr.11 0) val.10))
                                  (mref curr.11 -8)
                                  (fglt-help$3 v.9 val.10 (+ curr.11 8)
                                               size.12))))])
      (locals (v.13)
        (begin
          (set! v.13 (chained-vector-set!$1
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
                       9 90))
          (find-greatest-less-than$2 v.13 76))))";
    test_helper(s, "c19.s", 70);
}

#[test]
fn compile20() {
    let s = "
    (letrec ([make-vector$0 (lambda (size.1)
                              (locals (v.20)
                                (begin
                                  (set! v.20 (alloc (* (+ size.1 1) 8)))
                                  (mset! 0 v.20 size.1)
                                  v.20)))]
             [vector-set!$1 (lambda (vect.2 off.3 val.4)
                              (locals ()
                                (begin
                                  (if (> off.3 (mref vect.2 0))
                                      (nop)
                                      (mset! (* (+ off.3 1) 8) vect.2 val.4))
                                  0)))]
             [vector-equal?$3 (lambda (vect1.8 vect2.9)
                                (locals ()
                                  (if (= (mref 0 vect1.8) (mref 0 vect2.9))
                                      (vector-equal?$4 vect1.8 vect2.9
                                                       (- (mref 0 vect1.8) 1))
                                      0)))]
             [vector-equal?$4 (lambda (vect1.11 vect2.12 off.10)
                                (locals ()
                                  (if (< off.10 0)
                                      1 
                                      (if (= (mref (* (+ off.10 1) 8) vect1.11)
                                             (mref vect2.12 (* (+ off.10 1) 8)))
                                          (vector-equal?$4 vect1.11 vect2.12
                                                           (- off.10 1))
                                          0))))])
      (locals (v1.13 v2.14)
        (begin
          (set! v1.13 (make-vector$0 5))
          (vector-set!$1 v1.13 0 134)
          (vector-set!$1 v1.13 1 123)
          (vector-set!$1 v1.13 2 503)
          (vector-set!$1 v1.13 3 333)
          (vector-set!$1 v1.13 4 666)
          (set! v2.14 (make-vector$0 5))
          (vector-set!$1 v2.14 0 134)
          (vector-set!$1 v2.14 1 123)
          (vector-set!$1 v2.14 2 503)
          (vector-set!$1 v2.14 3 333)
          (vector-set!$1 v2.14 4 666)
          (if (= (vector-equal?$3 v1.13 v2.14) 0)
              100
              -100))))";
    test_helper(s, "c20.s", -100);
}

#[test]
#[should_panic()]
fn compile21() {
    let s = "
       (letrec ([stack-new$0 (lambda (size.1)
                            (locals (stack.2 store.3 meths.4)
                              (begin
                                (set! store.3 (alloc (* 8 size.1)))
                                (set! meths.4 (alloc (* 3 8)))
                                (set! stack.2 (alloc (* 3 8)))
                                (mset! meths.4 0 stack-push$2)
                                (mset! meths.4 8 stack-pop$3)
                                (mset! meths.4 16 stack-top$4)
                                (mset! stack.2 0 meths.4)
                                (mset! stack.2 8 0)
                                (mset! stack.2 16 store.3)
                                stack.2)))]
             [invoke$1 (lambda (obj.5 meth-idx.6)
                         (locals ()
                           (mref (mref obj.5 0) (* meth-idx.6 8))))]
             [stack-push$2 (lambda (self.7 val.8)
                             (locals ()
                               (begin
                                 (mset! (mref self.7 16) 
                                        (* (mref self.7 8) 8)
                                        val.8)
                                 (mset! self.7 8 (+ (mref self.7 8) 1))
                                 self.7)))]
             [stack-pop$3 (lambda (self.9)
                            (locals ()
                              (begin
                                (mset! self.9 8 (- (mref 8 self.9) 1))
                                (mref (mref self.9 16) 
                                      (* (mref self.9 8) 8)))))]
             [stack-top$4 (lambda (self.9)
                            (locals ()
                              (mref (mref self.9 16) 
                                    (* (- (mref 8 self.9) 1) 8))))])
      (locals (s1.10 s2.11 x.1000 x.1001 x.1002)
        (begin
          (set! s1.10 (stack-new$0 10))
          ((invoke$1 s1.10 0) s1.10 10) ;; push 10
          ((invoke$1 s1.10 0) s1.10 20) ;; push 20
          ((invoke$1 s1.10 0) s1.10 30) ;; push ... well you get the idea
          ((invoke$1 s1.10 0) s1.10 40)
          ((invoke$1 s1.10 0) s1.10 50)
          ((invoke$1 s1.10 0) s1.10 60)
          ((invoke$1 s1.10 0) s1.10 70)
          ((invoke$1 s1.10 0) s1.10 80)
          ((invoke$1 s1.10 0) s1.10 90)
          ((invoke$1 s1.10 0) s1.10 100)
          (set! s2.11 (stack-new$0 6))
          ((invoke$1 s2.11 0) s2.11 ((invoke$1 s1.10 1) s1.10)) ;; push pop
          ((invoke$1 s1.10 1) s1.10) ;; pop
          ((invoke$1 s2.11 0) s2.11 ((invoke$1 s1.10 1) s1.10))
          ((invoke$1 s1.10 1) s1.10) ;; pop
          ((invoke$1 s2.11 0) s2.11 ((invoke$1 s1.10 1) s1.10))
          ((invoke$1 s1.10 1) s1.10) ;; pop
          ((invoke$1 s2.11 0) s2.11 ((invoke$1 s1.10 1) s1.10))
          ((invoke$1 s1.10 1) s1.10) ;; pop
          ((invoke$1 s2.11 0) s2.11 ((invoke$1 s1.10 1) s1.10))
          ((invoke$1 s2.11 0) s2.11 ((invoke$1 s1.10 1) s1.10))
          (set! x.1000 (+ ((invoke$1 s2.11 1) s2.11) ((invoke$1 s2.11 1) s2.11)))
          (* (+ (begin
                  (set! x.1001 (+ ((invoke$1 s2.11 2) s2.11) ((invoke$1 s2.11 2) s2.11)))
                  (- x.1001 (+ ((invoke$1 s2.11 1) s2.11) ((invoke$1 s2.11 1) s2.11))))
                (begin
                  (set! x.1002 (+ ((invoke$1 s2.11 2) s2.11) ((invoke$1 s2.11 2) s2.11)))
                  (- (+ ((invoke$1 s2.11 1) s2.11) ((invoke$1 s2.11 1) s2.11)) x.1002)))
             x.1000))))";
    test_helper(s, "c21.s", 1);
}

#[test]
fn compile22() {
    let s = "
    (letrec ([add$0
               (lambda (x.1 y.2)
                 (locals (z.3)
                   (begin
                     (set! z.3 (alloc 8))
                     (mset! z.3 0 (+ x.1 y.2))
                     z.3)))])
      (locals()
        (mref (add$0 1 2) 0)))";
    test_helper(s, "c22.s", 3);
}

#[test]
fn compile23() {
    let s = "
    (letrec ([d$1 (lambda ()
                    (locals () (alloc 16)))])
      (locals (b.2 c.3)
        (begin
          (set! b.2 32)
          (set! c.3 (d$1))
          (mset! c.3 8 b.2)
          (mref c.3 8))))";
    test_helper(s, "c23.s", 32);
}

#[test]
fn compile24() {
    let s = "
    (letrec ([add-one$0 (lambda (x.0) (locals () (+ x.0 1)))]
             [sum-add-one-twice$1 (lambda (x.0)
                                    (locals ()
                                      (+ (add-one$0 x.0) (add-one$0 x.0))))])
      (locals () (sum-add-one-twice$1 1)))";
    test_helper(s, "c24.s", 4);
}


#[test]
fn compile25() {
    let s = "
    (letrec ([cc$0 (lambda (fst.1 snd.2)
                     (locals (ptr.3)
                       (begin
                         (set! ptr.3 (alloc 16))
                         (mset! ptr.3 0 fst.1)
                         (mset! ptr.3 8 snd.2)
                         ptr.3)))]
             [add1$3 (lambda (n.6) (locals () (+ n.6 1)))]
             [map$4 (lambda (f.7 ls.8)
                      (locals (t.100 s.101 q.102)
                        (if (= ls.8 0)
                            0
                            (begin
                                (set! t.100 (mref ls.8 0))
                                (set! q.102 (mref ls.8 8))
                                (set! s.101 (+ 1 t.100))
                                (cc$0 s.101 (map$4 f.7 q.102))))))])
      (locals (ls.10 r.11)
        (begin
          (set! ls.10 (cc$0 1 0))
          (set! r.11 (map$4 add1$3 ls.10))
          (mref r.11 0))))";
    test_helper(s, "c25.s", 2);
}

#[test]
fn compile26() {
    let s = "
    (letrec ([add1$3 (lambda () (locals () 1))]
             [high$4 (lambda (f.7)
                      (locals ()
                        (begin
                            (f.7)
                            (f.7))))])
      (locals ()
        (begin
          (high$4 add1$3))))";
    test_helper(s, "c26.s", 1);
}
