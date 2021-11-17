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
    let s = "(letrec () (let ([n.1 '19]) (if (<= n.1 '19) '#t '#f)))";
    test_helper(s, "c1-1.s", "#t");
    let s = "(letrec () (let ([n.1 '#f]) (if (eq? n.1 n.1) '() '-1)))";
    test_helper(s, "c1-2.s", "()");

}


#[test]
fn compile2() {
    let s = "(letrec () (let ([n.1 '#t]) (if (eq? n.1 '17) '() '-1)))";
    test_helper(s, "c2.s", "-1");
}

#[test]
fn compile3() {
    let s = "(letrec () (let ([n.1 (make-vector '3)]) (if (eq? n.1 '17) '() '-1)))";
    test_helper(s, "c3.s", "-1");
}

#[test]
fn compile4() {
    let s = "(letrec () (let ([n.1 '#f]) (if (eq? n.1 (void)) '() '-1)))";
    test_helper(s, "c4.s", "-1");
}

#[test]
fn compile5() {
    let s = "(letrec () (let ([n.1 '#f]) (if (null? n.1) '5 '-7)))";
    test_helper(s, "c5.s", "-7");
}


#[test]
fn compile6() {
    let s = "
    (letrec ([add1$3 (lambda (n.6) (+ n.6 '1))]
             [map$4 (lambda (f.7 ls.8)
                      (if (null? ls.8)
                          '()
                          (cons (f.7 (car ls.8)) 
                                (map$4 f.7 (cdr ls.8)))))]
             [sum$5 (lambda (ls.9)
                      (if (null? ls.9)
                          '0
                          (+ (car ls.9) (sum$5 (cdr ls.9)))))])
      (let ([ls.10 (cons '5 (cons '4 (cons '3 (cons '2 (cons '1 '())))))])
        (let ([ls.11 (cons '10 (cons '9 (cons '8 (cons '7 (cons '6 ls.10)))))])
          (sum$5 (map$4 add1$3 ls.11)))))";
    test_helper(s, "c6.s", "65");
}


#[test]
fn compile7() {
    let s = "
    (letrec ([thunk-num$0 (lambda (n.1)
                            (let ([th.2 (make-vector '2)])
                              (begin 
                                (vector-set! th.2 '0 force-th$1)
                                (vector-set! th.2 '1 n.1)
                                th.2)))]
             [force-th$1 (lambda (cl.3)
                           (vector-ref cl.3 '1))]
             [add-ths$2 (lambda (cl1.4 cl2.5 cl3.6 cl4.7)
                          (+ (+ ((vector-ref cl1.4 '0) cl1.4)
                                ((vector-ref cl2.5 '0) cl2.5))
                             (+ ((vector-ref cl3.6 '0) cl3.6)
                                ((vector-ref cl4.7 '0) cl4.7))))])
      (add-ths$2 (thunk-num$0 '5) (thunk-num$0 '17) (thunk-num$0 '7)
                 (thunk-num$0 '9)))";
    test_helper(s, "c7.s", "38");
}

#[test]
fn compile8() {
    let s = "
    (letrec ([gcd$0 (lambda (x.1 y.2)
                      (if (= y.2 '0) 
                          x.1 
                          (gcd$0 (if (> x.1 y.2) (- x.1 y.2) x.1)
                                 (if (> x.1 y.2) y.2 (- y.2 x.1)))))])
      (gcd$0 '1071 '1029))";
    test_helper(s, "c8.s", "21");
}

#[test]
fn compile9() {
    let s = "
    (letrec ([if-test$5 (lambda (n.1 x.2 y.3)
                          (begin
                            (if (= n.1 '0)
                                (vector-set! x.2 '0 (+ (vector-ref x.2 '0) (vector-ref y.3 '0)))
                                (vector-set! y.3 '0 (+ (vector-ref y.3 '0) (vector-ref x.2 '0))))
                            (vector-set! x.2 '0 (+ (vector-ref x.2 '0) n.1))
                            (if (if (= n.1 (vector-ref y.3 '0)) (false) (true))
                                (+ n.1 (vector-ref x.2 '0))
                                (+ n.1 (vector-ref y.3 '0)))))])
       (let ([q.6 (make-vector '1)] [p.7 (make-vector '1)])
         (begin
           (vector-set! q.6 '0 '1)
           (vector-set! p.7 '0 '2)
           (if-test$5 '3 q.6 p.7))))";
    test_helper(s, "c9.s", "6");
}

#[test]
fn compile10() {
    let s = "
    (letrec ()
      (let ([v (make-vector '3)])
        (begin 
          (vector-set! v '0 '10)
          (vector-set! v '1 '2)
          (vector-set! v '2 '4)
          (vector-ref v '1))))";
    test_helper(s, "c10.s", "2");
}

#[test]
fn compile11() {
    let s = "
    (letrec ()
      (let ([p (cons '10 '20)])
        (begin 
          (set-car! p '42)
          (cdr p))))";
    test_helper(s, "c11.s", "20");
}