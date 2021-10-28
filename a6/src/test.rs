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
    (letrec ([main$0 (lambda (a.1 b.2 d.5)
                       (locals (c.3)
                         (begin
                           (set! c.3 
                             (if (if (= a.1 1) (true) (= b.2 1))
                                 1
                                 0))
                           (+ c.3 5))))])
      (locals () (main$0 0 1 2)))";
    let filename = "c1.s";
    compile(s, filename);
    let r = run_helper(filename);
    assert_eq!(r.as_str(), "6\n");
}

#[test]
fn compile2() {
    let s = " 
    (letrec ()
      (locals (a.1)
        (begin
          (set! a.1 10)
          (if (< 7 a.1)
              (nop)
              (set! a.1 (+ a.1 a.1)))
          a.1)))";
    let filename = "c2.s";
    compile(s, filename);
    let r = run_helper(filename);
    assert_eq!(r.as_str(), "10\n");
}

#[test]
fn compile3() {
    let s = "
    (letrec ()
      (locals (n.1 a.2 b.3 c.4)
        (begin
          (set! n.1 1)
          (begin
            (set! a.2 2)
            (begin
              (set! b.3 3)
              (set! n.1 (+ n.1 (if (= (+ n.1 b.3) b.3) 5 10)))
              (set! n.1 (+ n.1 b.3)))
            (set! n.1 (+ n.1 a.2)))
          (+ n.1 n.1))))";
    let filename = "c3.s";
    compile(s, filename);
    let r = run_helper(filename);
    assert_eq!(r.as_str(), "32\n");
}