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
fn compile1() {
    let s = 
    "(letrec ()
      (begin
        (set! rax 5)
        (begin
          (set! rbx 1)
          (begin
            (set! rbx (* rbx rax))
            (begin
              (set! rax (- rax 1))
              (begin
                (set! rbx (* rbx rax))
                (begin
                  (set! rax (- rax 1))
                  (begin
                    (set! rbx (* rbx rax))
                    (begin
                      (set! rax (- rax 1))
                      (begin
                        (set! rbx (* rbx rax))
                        (begin
                          (set! rax rbx)
                          (r15))))))))))))";
    let filename = "c1.s";
    compile(s, filename);
    let r = run_helper(filename);
    assert_eq!(r.as_str(), "120\n");
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

#[test]
fn compile3() {
    let s = "
    (letrec ([return$1 (lambda ()
                         (begin
                           (set! rax fv0)
                           (fv1)))]
             [setbit3$0 (lambda ()
                          (begin
                            (set! fv0 (logor fv0 8))
                            (return$1)))])
      (begin
        (set! fv0 1)
        (set! fv1 r15)
        (setbit3$0)))";
    let filename = "c3.s";
    compile(s, filename);
    let r = run_helper(filename);
    assert_eq!(r.as_str(), "9\n");
}

#[test]
fn compile4() {
    let s = "
    (letrec ((loop$1 (lambda () (begin
                                  (set! rax (+ rax rbx))
                                  (set! rbx r11)
                                  (set! r11 rax)
                                  (controller$2))))
             (controller$2 (lambda () (begin
                                        (set! rdx rcx)
                                        ; shift rdx righ by 63
                                        ; output of this is (in 2's comp.)
                                        ; 0 if rdx is zero or positive
                                        ; else -1. 
                                        (set! rdx (sra rdx 63))
                                        ; this will result either 0 or 8
                                        ; 8 if rdx was negative
                                        (set! rdx (logand rdx 8))
                                        (set! rbp (+ rbp rdx))
                                        ; fv0 may either be the original fv0
                                        ; or original fv1 (if rdx was neg.).
                                        ; original fv1 has the exit address.
                                        (set! r15 fv0)
                                        ; reset rbp
                                        (set! rbp (- rbp rdx))
                                        ; reduce rcx by 1
                                        (set! rcx (- rcx 1))
                                        (r15)))))
      (begin
        ; set initial values
        (set! rax 1)
        (set! rbx 0)
        (set! r11 0)
        ; keep the address referred by label loop$1 in fv0
        (set! rdx loop$1)
        (set! fv0 rdx)
        ; keep the return address in fv1
        (set! rdx r15)
        (set! fv1 rdx)
        ; the number we are interested in (e.g.Fib(10))
        (set! rcx 10)
        ; jump to the controller
        (controller$2)))))";
    let filename = "c4.s";
    compile(s, filename);
    let r = run_helper(filename);
    assert_eq!(r.as_str(), "89\n");
}


#[test]
fn compile5 () {
    let s = 
    "(letrec ((fact$0 (lambda ()
                       (begin
                         ; no if, so use a computed goto
                         ; put address of fact$1 at bfp[0]
                         (set! rcx fact$1)
                         (set! fv0 rcx)
                         ; put address of fact$2 at bfp[8]
                         (set! rcx fact$2)
                         (set! fv1 rcx)
                         ; if x == 0 set rcx to 8, else set rcx to 0
                         (set! rdx 0)
                         (set! rdx (- rdx rax))
                         (set! rdx (sra rdx 63))
                         (set! rdx (logand rdx 8))
                         ; point bfp at stored address of fact$1 or fact$2
                         (set! rbp (+ rbp rdx))
                         ; grab whichever and reset bfp
                         (set! rcx fv0)
                         (set! rbp (- rbp rdx))
                         ; tail call (jump to) fact$1 or fact$2
                         (rcx))))
             (fact$1 (lambda ()
                       (begin
                         ; get here if rax is zero, so return 1
                         (set! rax 1)
                         (r15))))
             (fact$2 (lambda ()
                       (begin
                         ; get here if rax is nonzero, so save return
                         ; address and eax, then call fact$0 recursively
                         ; with eax - 1, setting fact$3 as return point
                         (set! fv0 r15)
                         (set! fv1 rax)
                         (set! rax (- rax 1))
                         (set! r15 fact$3)
                         ; bump rbp by 16 (two 64-bit words) so that
                         ; recursive call doesn't wipe out our saved
                         ; eax and return address
                         (set! rbp (+ rbp 16))
                         (fact$0))))
             (fact$3 (lambda ()
                       (begin
                         ; restore rbp to original value
                         (set! rbp (- rbp 16))
                         ; eax holds value of recursive call, multiply
                         ; by saved value at fv1 and return to saved
                         ; return address at fv0
                         (set! rax (* rax fv1))
                         (fv0)))))
      (begin
        (set! rax 10)
        (fact$0)))";
    let filename = "c5.s";
    compile(s, filename);
    let r = run_helper(filename);
    assert_eq!(r.as_str(), "3628800\n");
}


#[test]
fn compile6() {
    let s = "
    (letrec ((div$0 (lambda ()
                      (begin
                        (set! fv2 (sra fv2 1))
                        (div$1))))
             (div$1 (lambda ()
                      (begin
                        (set! rax fv2)
                        (fv0)))))
      (begin
        (set! fv0 r15)
        (set! rax div$0)
        (set! fv1 rax)
        (set! fv2 64)
        (fv1)))";
    let filename = "c6.s";
    compile(s, filename);
    let r = run_helper(filename);
    assert_eq!(r.as_str(), "32\n");
}

#[test]
fn compile7() {
    let s = "
    (letrec ()
      (begin
        (set! r11 10)
        (set! r11 (* r11 -10))
        (set! rax r11)
        (r15)))";
    let filename = "c7.s";
    compile(s, filename);
    let r = run_helper(filename);
    assert_eq!(r.as_str(), "-100\n");
}

#[test]
fn compile8() {
    let s = "
    (letrec ([double$0 (lambda ()
                         (begin
                           (set! rax (+ rax rax))
                           (r15)))])
      (begin
        (set! rax 10)
        (double$0)))";
    let filename = "c8.s";
    compile(s, filename);
    let r = run_helper(filename);
    assert_eq!(r.as_str(), "20\n");
}

#[test]
fn compile9() {
    let s = "
    (letrec ([double$1 (lambda ()
                         (begin
                           (set! rax (+ rax rax))
                           (sqr$1)))]
             [sqr$1 (lambda ()
                      (begin
                        (set! rax (* rax rax))
                        (r15)))])
      (begin
        (set! rax 2)
        (double$1)))";
    let filename = "c9.s";
    compile(s, filename);
    let r = run_helper(filename);
    assert_eq!(r.as_str(), "16\n");
}