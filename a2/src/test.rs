use std::process::Command;
use crate::parser::{Scanner, Parser};
use crate::compiler::compile;
use crate::syntax::Expr;

fn test_token_helper(s: &str, r: Vec<&str>) -> bool {
    let scanner = Scanner::new(s);
    let tokens = scanner.scan();
    println!("{:?}", tokens);
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
fn token2() {
    let s = "(begin (set! rax -8))";
    let r = vec!["(", "begin", "(", "set!", "rax", "-8", ")", ")"];
    assert!(test_token_helper(s, r));
}

#[test]
fn token3() {
    let s = "(begin (set! rax -8))";
    let r = vec!["(", "begin", "(", "set!", "rax", "-8", ")", ")"];
    assert!(test_token_helper(s, r));
}


fn test_parser_helper(s: &str) {
    let scanner = Scanner::new(s);
    let tokens = scanner.scan();
    let parser = Parser::new(tokens);
    let ast = format!("{}", parser.parse());
    let ast_str: Vec<&str> = ast.split_whitespace().collect();
    let s_str: Vec<&str> = s.split_whitespace().collect();
    assert_eq!(s_str.join(""), ast_str.join(""));
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

#[test]
fn parse2() {
    use Expr::*;
    let s = "(begin (set! rax -10))";
    let scanner = Scanner::new(s);
    let tokens = scanner.scan();
    let parser = Parser::new(tokens);
    let ast = parser.parse();
    if let Begin(mut stats) = ast {
        let stat = stats.pop().unwrap();
        assert!(matches!(stat, Set(box Symbol(s), box Int64(-10))));
    }
}

#[test]
fn parse3() {
    let s = "(letrec () (r15))";
    test_parser_helper(s);
}


#[test]
fn parse4() {
    let s = "(letrec () (begin (set! rax 0) (r15)))";
    test_parser_helper(s);
}


#[test]
fn parse5() {
    let s = "(letrec ((f$1 (lambda () (begin 
                                        (set! fv0 rax)
                                        (set! rax (+ rax rax))
                                        (set! rax (+ rax fv0))
                                        (r15)))))
                (begin 
                    (set! rax 17)
                    (f$1)))";
    let scanner = Scanner::new(s);
    let tokens = scanner.scan();
    let parser = Parser::new(tokens);
    let ast = format!("{}", parser.parse());
    let ast_str: Vec<&str> = ast.split_whitespace().collect();
    let s_str: Vec<&str> = s.split_whitespace().collect();
    assert_eq!(s_str.join(""), ast_str.join(""));
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
fn compile2() {
    let s = "(letrec ()
                (begin
                    (set! rax 5)
                    (set! rbx 1)
                    (set! rbx (* rbx rax))
                    (set! rax (- rax 1))
                    (set! rbx (* rbx rax))
                    (set! rax (- rax 1))
                    (set! rbx (* rbx rax))
                    (set! rax (- rax 1))
                    (set! rbx (* rbx rax))
                    (set! rax rbx)
                    (r15)))";
    let filename = "c2.s";
    compile(s, filename);
    let r = run_helper(filename);
    assert_eq!(r.as_str(), "120\n");
}