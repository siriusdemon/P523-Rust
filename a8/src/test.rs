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