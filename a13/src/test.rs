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
    ((lambda (y.2)
        ((lambda (f.1) (f.1 (f.1 y.2)))
            (lambda (x.3) (+ x.3 '1))))
     '3)";
    test_helper(s, "c1.s", "5");
}

#[test]
fn compile2() {
    let s = "
    (let ([a.1 (lambda (x.2)
                 (lambda (y.3)
                   (* x.2 (- y.3 '1))))])
      (let ([b.4 (lambda (w.5)
                   ((a.1 '2) '4))])
        (b.4 '6)))";
    test_helper(s, "c2.s", "6");
}