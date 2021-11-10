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