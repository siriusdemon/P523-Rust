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
    assert_eq!(r.as_str().trim(), expect);
}

#[test]
fn compile1() {
    let s = "
    (let ([a.1 (letrec ([f$0 (lambda () '80)]) (f$0))]
          [b.2 (letrec ([g$1 (lambda () '50)]) (g$1))])
      (* a.1 b.2))";
    test_helper(s, "c21.s", "4000");
}

#[test]
fn compile2() {
    let s = "(let ([x.1 (cons '1 '5)]) (begin (car x.1) x.1))";
    test_helper(s, "c22.s", "(1 . 5)");
}

#[test]
fn compile3() {
    let s = "
    (letrec ()
      (let ([a (cons '1 '2)])
        a))";
    test_helper(s, "c3.s", "(1 . 2)");
}

#[test]
fn compile4() {
    let s = "(void)";
    test_helper(s, "c4.s", "#<void>");
}

#[test]
fn compile5() {
    let s = "
    (letrec ()
      (let ([a (cons '2 (cons '1 '()))])
        a))";
    test_helper(s, "c5.s", "(2 1)");
}


#[test]
fn compile6() {
    let s = "(letrec () (begin (make-vector '5) '7))";
    test_helper(s, "c6.s", "7");
}

#[test]
fn compile7() {
    let s = "(if (cdr (cons '#t '#f)) '7 '8)";
    test_helper(s, "c7.s", "8");
}

#[test]
fn compile8() {
    let s = "(letrec () (if (make-vector '10) '7 '8))";
    test_helper(s, "c8-1.s", "7");
    let s = "
    (letrec () 
      (let ([v.1 (make-vector '10)])
        (if (vector-length v.1) '7 '8)))";
    test_helper(s, "c8-2.s", "7");
    let s = "
    (letrec () 
      (let ([v.1 (make-vector '10)])
        (begin
          (vector-set! v.1 '0 '#t)
          (if (vector-ref v.1 '0) '7 '8))))";
    test_helper(s, "c8-3.s", "7");
}

#[test]
fn compile9() {
    let s = "    
    (letrec () 
      (let ([x.1 (cons '1 '())] [y.2 (cons '1 '())])
        (eq? x.1 y.2)))";
    test_helper(s, "c9-1.s", "#f");
    let s = "(vector? (make-vector '1))";
    test_helper(s, "c9-2.s", "#t");
}

#[test]
fn compile10() {
    let s = "(letrec () (begin (boolean? '#f) '9))";
    test_helper(s, "c10-1.s", "9");
    let s = "    
    (letrec () 
      (let ([x.1 (cons '1 '())] [y.2 (cons '1 '())])
        (begin (eq? x.1 y.2) '10)))";
    test_helper(s, "c10-2.s", "10");
    let s = "(letrec () (begin (null? '()) '15))";
    test_helper(s, "c10-3.s", "15");
    let s = "(letrec () (begin (pair? (cons '1 '())) '20))";
    test_helper(s, "c10-4.s", "20");
}

#[test]
fn compile11() {
    let s = "(letrec () (vector-set! (make-vector '4) '0 '10))";
    test_helper(s, "c11-1.s", "#<void>");
    let s = "(letrec () (set-car! (cons '1 '2) '10))";
    test_helper(s, "c11-2.s", "#<void>");
    let s = "(letrec () (set-cdr! (cons '1 '2) '14))";
    test_helper(s, "c11-3.s", "#<void>");
}