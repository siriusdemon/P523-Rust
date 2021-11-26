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
fn compile1_1() {
    let s = "'7";
    test_helper(s, "1-1.s", "7");
    let s = "'()";
    test_helper(s, "1-2.s", "()");
    let s = "'#f";
    test_helper(s, "1-3.s", "#f");
    let s = "'(1 2 3 4)";
    test_helper(s, "1-4.s", "(1 2 3 4)");
}

#[test]
fn compile1_2() {
    let s = "'#5(5 4 3 2 1)";
    test_helper(s, "1-5.s", "#(5 4 3 2 1)");
    let s = "'#2((1 2) (3 4))";
    test_helper(s, "1-6.s", "#((1 2) (3 4))");
    let s = "'(#2(1 2) #2(3 4))";
    test_helper(s, "1-7.s", "(#(1 2) #(3 4))");
    let s = "'(#3(#t #f 1) #3(#f #t 2))";
    test_helper(s, "1-8.s", "(#(#t #f 1) #(#f #t 2))");
}

#[test]
fn compile1_3() {
    let s = "(let ([t.496 '10]) (if t.496 t.496 '#f))";
    test_helper(s, "1-9.s", "10");
    let s = "(if '#t (if '45 '7 '#f) '#f)";
    test_helper(s, "1-10.s", "7");
    let s = "(+ '4 '5)";
    test_helper(s, "1-11.s", "9");
    let s = "(- '1 '4)";
    test_helper(s, "1-12.s", "-3");
    let s = "(* '7 '9)";
    test_helper(s, "1-13.s", "63");
}

#[test]
fn compile1_4() {
    let s = "(cons '1 '())";
    test_helper(s, "1-14.s", "(1)");
    let s = "(car '(1 2))";
    test_helper(s, "1-15.s", "1");
    let s = "(cdr '(1 2))";
    test_helper(s, "1-16.s", "(2)");
    let s = "(if '#t '1 '2)";
    test_helper(s, "1-17.s", "1");
    let s = "(pair? '(1 2))";
    test_helper(s, "1-18.s", "#t");
    let s = "(pair? '())";
    test_helper(s, "1-19.s", "#f");
    let s = "(vector? '#2(1 2))";
    test_helper(s, "1-20.s", "#t");
    let s = "(vector? '(1 2))";
    test_helper(s, "1-21.s", "#f");
    let s = "(boolean? '#f)";
    test_helper(s, "1-22.s", "#t");
    let s = "(boolean? '7)";
    test_helper(s, "1-23.s", "#f");
    let s = "(null? '())";
    test_helper(s, "1-24.s", "#t");
    let s = "(null? '(1 2))";
    test_helper(s, "1-25.s", "#f");
    let s = "(fixnum? '1234)";
    test_helper(s, "1-26.s", "#t");
    let s = "(fixnum? '())";
    test_helper(s, "1-27.s", "#f");
    let s = "(procedure? (lambda (x.495) x.495))";
    test_helper(s, "1-28.s", "#t");
    let s = "(procedure? '7)";
    test_helper(s, "1-29.s", "#f");
}

#[test]
fn compile1_5() {
    let s = "(<= '1 '8)";
    test_helper(s, "1-30.s", "#t");
    let s = "(<= '8 '1)";
    test_helper(s, "1-31.s", "#f");
    let s = "(<= '1 '1)";
    test_helper(s, "1-32.s", "#t");
    let s = "(< '8 '1)";
    test_helper(s, "1-33.s", "#f");
    let s = "(< '1 '8)";
    test_helper(s, "1-34.s", "#t");
    let s = "(= '1 '1)";
    test_helper(s, "1-35.s", "#t");
    let s = "(= '1 '0)";
    test_helper(s, "1-36.s", "#f");
    let s = "(>= '8 '1)";
    test_helper(s, "1-37.s", "#t");
    let s = "(>= '1 '8)";
    test_helper(s, "1-38.s", "#f");
    let s = "(>= '1 '1)";
    test_helper(s, "1-39.s", "#t");
    let s = "(> '8 '1)";
    test_helper(s, "1-40.s", "#t");
    let s = "(> '1 '8)";
    test_helper(s, "1-41.s", "#f");
    let s = "(if '#f '#f '#t)";
    test_helper(s, "1-42.s", "#t");
    let s = "(if '10 '#f '#t)";
    test_helper(s, "1-43.s", "#f");
}

#[test]
fn compile2() {
    let s = "
    (let ([f.1 (lambda () '(1 . 2))])
        (eq? (f.1) (f.1)))";
    test_helper(s, "c2.s", "#t");
}

#[test]
fn compile3() {
    let s = "
    (let ([f.1 (lambda () '#2(1 2))])
        (eq? (f.1) (f.1)))";
    test_helper(s, "c3.s", "#t");
}

#[test]
fn compile4() {
    let s = "
    (let ([f.1 (lambda () '(#2(1 2) #2(1 2) (1 2)))])
        (eq? (f.1) (f.1)))";
    test_helper(s, "c4.s", "#t");
}

#[test]
fn compile5() {
    let s = "
    (letrec ([f.1 (lambda (x.2) 
                    (begin
                      (set! f.1 x.2)
                      f.1))])
      (f.1 '10))";
    test_helper(s, "c5.s", "10");
}

#[test]
fn compile6() {
    let s = "    
    (letrec ([x.1 (lambda () (begin (set! x.1 '2) '1))])
      (let ([y.2 (x.1)])
        (let ([z.3 x.1])
          (cons y.2 z.3))))";
    test_helper(s, "c6.s", "(1 . 2)");
}

#[test]
fn compile7_1() {
    let s = "
    (letrec ([x.1 (lambda () (f.2))]
             [f.2 (lambda () '10)])
      (x.1))";
    test_helper(s, "c7-1.s", "10");
}


#[test]
fn compile7_2() {
    let s = "
    (letrec ([x.1 (lambda () (begin (set! f.2 '10) f.2))]
             [f.2 (lambda () '10)])
      (x.1))";
    test_helper(s, "c7-2.s", "10");
}

#[test]
fn compile8_useful_for_uncover_assigned() {
    let s = "
    (let ([x.3 '10] [y.1 '11] [z.2 '12])
      (let ([f.9 (lambda (u.7 v.6)
                    (begin
                      (set! x.3 u.7)
                      (+ x.3 v.6)))]
            [g.8 (lambda (r.5 s.4)
                    (begin
                      (set! y.1 (+ z.2 s.4))
                      y.1))])
        (* (f.9 '1 '2) (g.8 '3 '4))))";
    test_helper(s, "c8-1.s", "48");
    let s = "
    (let ([x.3 '10] [y.1 '11] [z.2 '12])
      (let ([f.7 '#f]
            [g.6 (lambda (r.5 s.4)
                    (begin
                      (set! y.1 (+ z.2 s.4))
                        y.1))])
        (begin
          (set! f.7 (lambda (u.9 v.8)
                        (begin
                          (set! v.8 u.9)
                          (+ x.3 v.8))))
          (* (f.7 '1 '2) (g.6 '3 '4)))))";
    test_helper(s, "c8-2.s", "176");
}

#[test]
fn compile9() {
    let s = "    
    (letrec ([filter.1 (lambda (pred?.2 ls.3)
                         (if (null? ls.3) 
                             '()
                             (if (pred?.2 (car ls.3))
                                 (filter.1 pred?.2 (cdr ls.3))
                                 (cons (car ls.3) 
                                       (filter.1 pred?.2 (cdr ls.3))))))])
      (filter.1 (lambda (x.4) (< x.4 '0)) '(3 -5 91 6 -32 8)))";
    test_helper(s, "c9.s", "(3 91 6 8)");
}

#[test]
fn compile10() {
    let s = "    
    (letrec ([add.0 (lambda (n.2)
                      (lambda (n.3)
                        (+ n.2 n.3)))]
             [map.1 (lambda (fn.4 ls.5)
                      (if (null? ls.5)
                          '()
                          (cons (fn.4 (car ls.5)) (map.1 fn.4 (cdr ls.5)))))]
             [map.9 (lambda (fn.10 fnls.11 ls.12)
                      (if (null? ls.12)
                          '()
                          (cons (fn.10 (car fnls.11) 
                                       (car ls.12)) 
                                (map.9 fn.10 (cdr fnls.11) (cdr ls.12)))))])
      (let ([ls.6 '(1 2 3 4 5 6)])
        (map.9 (lambda (fn.7 elem.8) (fn.7 elem.8)) 
            (map.1 add.0 ls.6) ls.6)))";
    test_helper(s, "c10.s", "(2 4 6 8 10 12)");
}


// v.4 '#3(0) is not supported in my impl
#[test]
#[should_panic()]
fn compile11() {
    let s = "    
    (let [(x.1 '(4))
          (y.2 '(1 2 3))
          (v.4 '#3(0))]
      (letrec [(z.3 (cons y.2 x.1))]
        (begin
          (vector-set! v.4 '0 z.3)
          (set! x.1 '(3))
          (vector-set! v.4 '1 z.3)
          (vector-set! v.4 '2 (cons y.2 x.1))
          v.4)))";
    test_helper(s, "c11.s", "");
}

#[test]
fn compile12() {
    let s = "
    (let ([x.1 '0])
      (begin
        (let ([x.2 '1])
          (begin 
            (set! x.1 (+ x.1 '1))
            (set! x.2 (+ x.2 x.1))
            x.2))))";
    test_helper(s, "c12.s", "2");
}

#[test]
fn compile13() {
    let s = "    
    (cons
      (let ([f.463 (lambda (h.462 v.461) (* h.462 v.461))])
        (let ([k.465 (lambda (x.464) (+ x.464 '5))])
          (letrec ([x.466 '15])
            (letrec ([g.467 (lambda (x.468) (+ '1 x.468))])
              (k.465 (g.467 (let ([g.469 '3]) (f.463 g.469 x.466))))))))
      '())";
    test_helper(s, "c13.s", "(51)");
}

#[test]
fn compile14() {
    let s = "
    (letrec ([x.1 '15])
      x.1)";
    test_helper(s, "c14.s", "15");
}