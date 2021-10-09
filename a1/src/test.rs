use std::process::Command;
use crate::parser::{Scanner, Parser};
use crate::compiler::compile;

fn test_token_helper(s: &str, r: Vec<&str>) -> bool {
    let scanner = Scanner::new(s);
    let tokens = scanner.scan();
    for (token, res) in tokens.into_iter().zip(r) {
        if token.token != res {
            return false;
        } 
    }
    return true;
}

#[test]
fn token1() {
    let s = "(begin (set! rax 8) (set! rcx 3) (set! rax (- rax rcx)))"; 
    let r = vec![
        "(", "begin", 
            "(", "set!", "rax", "8", ")", 
            "(", "set!", "rcx", "3", ")", 
            "(", "set!", "rax", "(", "-", "rax", "rcx", ")", ")", ")"];
    assert!(test_token_helper(s, r));
}

#[test]
fn parse1() {
    let s = "(begin (set! rax 8) (set! rcx 3) (set! rax (- rax rcx)))"; 
    let scanner = Scanner::new(s);
    let tokens = scanner.scan();
    let parser = Parser::new(tokens);
    let ast = format!("{}", parser.parse());
    let ast_str: Vec<&str> = ast.split_whitespace().collect();
    let s_str: Vec<&str> = s.split_whitespace().collect();
    assert_eq!(s_str, ast_str);
}

fn run_helper(filename: &str) -> String {
    let obj: Vec<&str> = filename.split(".").collect();
    let stem = format!("test_{}", &obj[0]);
    let output = Command::new("/usr/bin/gcc")
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
    let s = "(begin (set! rax 8) (set! rcx 3) (set! rax (- rax rcx)))"; 
    let filename = "c1.s";
    compile(s, filename);
    let r = run_helper(filename);
    assert_eq!(r.as_str(), "5\n");
}