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
    test_helper(s, "c9.s", 11);
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
    test_helper(s, "c12.s", 55);
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
