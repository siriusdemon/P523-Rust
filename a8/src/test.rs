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

fn test_helper(program: &str, filename: &str, expect: &str) {
    compile(program, filename);
    let r = run_helper(filename);
    assert_eq!(r.as_str(), expect);
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
    test_helper(s, "c1.s", "3\n");
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
    test_helper(s, "c2.s", "0\n");
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
    test_helper(s, "c3.s", "16\n");
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
    test_helper(s, "c4.s", "\n");
}