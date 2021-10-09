use crate::parser::{Scanner, Parser};


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