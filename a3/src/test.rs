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
      (locate ()
        (begin
          (set! rax 0)
          (set! rbx 1)
          (if (if (= rax 1) (= rbx 1) (true))
              (set! rax 1)
              (set! rax 0))
          (r15))))";
    let filename = "c1.s";
    compile(s, filename);
    let r = run_helper(filename);
    assert_eq!(r.as_str(), "1\n");
}

#[test]
fn compile2() {
    let s = "
    (letrec ([fib$0 (lambda ()
                      (locate ([n.1 rax] [a.2 rbx] [b.3 rcx])
                        (begin
                          (set! a.2 0)
                          (set! b.3 1)
                          (fib$1))))]
             [fib$1 (lambda ()
                      (locate ([n.1 rax] [a.2 rbx] [b.3 rcx] [t.4 fv1]
                               [return.5 rax])
                        (if (= n.1 0)
                            (begin
                              (set! return.5 a.2)
                              (fv0))
                            (begin
                              (set! n.1 (- n.1 1))
                              (set! t.4 a.2)
                              (set! a.2 b.3)
                              (set! b.3 (+ b.3 t.4))
                              (fib$1)))))])
      (locate ([n.1 rax])
        (begin
          (set! fv0 r15)
          (set! n.1 5)
          (fib$0))))";
    let filename = "c2.s";
    compile(s, filename);
    let r = run_helper(filename);
    assert_eq!(r.as_str(), "5\n");
}


#[test]
fn compile3() {
    let s = "
    (letrec ([if-test$6 (lambda ()
                          (locate ([n.1 rdi] [x.2 rax] [y.3 rbx])
                            (begin
                              (set! x.2 1)
                              (begin
                                (set! y.3 1)
                                (if (= n.1 0)
                                    (set! x.2 (+ x.2 y.3))
                                    (set! y.3 (+ y.3 x.2)))
                                (set! x.2 n.1))
                              (if (if (= n.1 y.3) (false) (true))
                                  (set! n.1 (+ n.1 x.2))
                                  (set! n.1 (+ n.1 y.3)))
                              (set! x.2 n.1)
                              (r15))))])
      (locate ([n.1 rdi])
        (begin
          (set! n.1 1)
          (if-test$6))))";
    let filename = "c3.s";
    compile(s, filename);
    let r = run_helper(filename);
    assert_eq!(r.as_str(), "2\n");
}

#[test]
fn compile4() {
    let s = "
    (letrec ()
      (locate ([n.1 rdi])
        (begin 
          (set! rax 10)
          (set! rbx 2)
          (if (< rax rbx)
              (set! rax 10)
              (set! rax 2))
          (r15))))";
    let filename = "c4.s";
    compile(s, filename);
    let r = run_helper(filename);
    assert_eq!(r.as_str(), "2\n");
}

#[test]
#[should_panic()]
fn compile5() {
    let s = "
    (letrec ()
      (locate ([n.1 rdi])
        (begin 
          (if (< 10 2)
              (set! rax 10)
              (set! rax 2))
          (r15))))";
    let filename = "c5.s";
    compile(s, filename);
    let r = run_helper(filename);
    assert_eq!(r.as_str(), "2\n");
}

#[test]
fn compile6() {
    let s = "
    (letrec ()
      (locate ([n.1 rdi])
        (begin 
          (set! rax 2)
          (if (< 10 rax)
              (set! rax 10)
              (set! rax 2))
          (r15))))";
    let filename = "c6.s";
    compile(s, filename);
    let r = run_helper(filename);
    assert_eq!(r.as_str(), "2\n");
}