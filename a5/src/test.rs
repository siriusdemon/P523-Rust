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


#[test]
fn compile1() {
    let s = "
    (letrec ()
       (locals (x.5 y.6)
         (begin
           (set! x.5 5)
           (set! y.6 6)
           (set! x.5 (+ x.5 y.6))
           (set! rax x.5)
           (r15 rax))))";
    let filename = "c1.s";
    compile(s, filename);
    let r = run_helper(filename);
    assert_eq!(r.as_str(), "11\n");
}

#[test]
fn compile2() {
    let s = "
     (letrec ([div$0 (lambda ()
                       (locals ()
                         (begin 
                           (set! fv2 (sra fv2 1)) 
                           (div$1 fv2 fv0 rbp))))]
              [div$1 (lambda ()
                       (locals ()
                         (begin 
                           (set! rax fv2) 
                           (fv0 rax rbp))))])
       (locals (label-temp.1)
         (begin
           (set! fv0 r15)
           (set! label-temp.1 div$0)
           (set! fv1 label-temp.1)
           (set! fv2 64)
           (fv1 fv0 fv2 rbp))))";
    let filename = "c2.s";
    compile(s, filename);
    let r = run_helper(filename);
    assert_eq!(r.as_str(), "32\n");
}


#[test]
fn compile3() {
    let s = "
    (letrec ()
       (locals ()
         (begin (set! rax 5) (set! rax (+ rax 10)) (r15 rax))))";
    let filename = "c3.s";
    compile(s, filename);
    let r = run_helper(filename);
    assert_eq!(r.as_str(), "15\n");
}

#[test]
fn compile4() {
    let s = "
    (letrec ([setbit3$0 (lambda ()
                           (locals ()
                             (begin
                               (set! fv0 (logor fv0 8))
                               (return$1 fv0 fv1 rbp))))]
              [return$1 (lambda ()
                          (locals ()
                            (begin 
                              (set! rax fv0)
                              (fv1 rax rbp))))])
       (locals ()
         (begin (set! fv0 1) (set! fv1 r15) (setbit3$0 fv0 fv1 rbp))))";
    let filename = "c4.s";
    compile(s, filename);
    let r = run_helper(filename);
    assert_eq!(r.as_str(), "9\n");
}


#[test]
fn compile5() {
    let s = "
    (letrec ([square$1 (lambda ()
                          (locals (x.1)
                            (begin
                              (set! x.1 fv0)
                              (set! x.1 (* x.1 x.1))
                              (set! rax x.1)
                              (r15 rbp rax))))])
       (locals ()
         (begin
           (set! fv0 7)
           (square$1 rbp r15 fv0))))";
    let filename = "c5.s";
    compile(s, filename);
    let r = run_helper(filename);
    assert_eq!(r.as_str(), "49\n");
}

#[test]
fn compile6() {
    let s = "
    (letrec ([if-test$1 (lambda ()
                           (locals (x.5)
                             (begin
                               (if (begin (set! x.5 5) (= x.5 5))
                                   (set! x.5 (+ x.5 10))
                                   (set! x.5 (- x.5 10)))
                               (set! x.5 (* x.5 10))
                               (set! rax x.5)
                               (r15 rax))))])
       (locals () (if-test$1 r15)))";
    let filename = "c6.s";
    compile(s, filename);
    let r = run_helper(filename);
    assert_eq!(r.as_str(), "150\n");
}

#[test]
fn compile7() {
    let s = "
    (letrec ()
       (locals (b.2 g.7 c.3 d.4 e.5 a.1 f.6)
         (begin
           (set! a.1 1)
           (set! b.2 2)
           (set! c.3 a.1)
           (set! d.4 4)
           (set! e.5 5)
           (set! f.6 b.2)
           (set! f.6 (+ f.6 c.3))
           (set! f.6 (+ f.6 d.4))
           (set! f.6 (+ f.6 e.5))
           (set! g.7 7)
           (set! f.6 (+ f.6 g.7))
           (set! rax f.6)
           (r15 rax rcx rdx rbx rbp rsi rdi r8 r9 r10 r11 r12))))";
    let filename = "c7.s";
    compile(s, filename);
    let r = run_helper(filename);
    assert_eq!(r.as_str(), "19\n");
}