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
