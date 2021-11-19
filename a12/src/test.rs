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


#[test]
fn compile6() {
    let s = "(letrec () (begin (make-vector '5) '7))";
    test_helper(s, "c6.s", "7");
}

#[test]
fn compile7() {
    let s = "(if (cdr (cons '#t '#f)) '7 '8)";
    test_helper(s, "c7.s", "8");
}

#[test]
fn compile8() {
    let s = "(letrec () (if (make-vector '10) '7 '8))";
    test_helper(s, "c8-1.s", "7");
    let s = "
    (letrec () 
      (let ([v.1 (make-vector '10)])
        (if (vector-length v.1) '7 '8)))";
    test_helper(s, "c8-2.s", "7");
    let s = "
    (letrec () 
      (let ([v.1 (make-vector '10)])
        (begin
          (vector-set! v.1 '0 '#t)
          (if (vector-ref v.1 '0) '7 '8))))";
    test_helper(s, "c8-3.s", "7");
}

#[test]
fn compile9() {
    let s = "    
    (letrec () 
      (let ([x.1 (cons '1 '())] [y.2 (cons '1 '())])
        (eq? x.1 y.2)))";
    test_helper(s, "c9-1.s", "#f");
    let s = "(vector? (make-vector '1))";
    test_helper(s, "c9-2.s", "#t");
}

#[test]
fn compile10() {
    let s = "(letrec () (begin (boolean? '#f) '9))";
    test_helper(s, "c10-1.s", "9");
    let s = "    
    (letrec () 
      (let ([x.1 (cons '1 '())] [y.2 (cons '1 '())])
        (begin (eq? x.1 y.2) '10)))";
    test_helper(s, "c10-2.s", "10");
    let s = "(letrec () (begin (null? '()) '15))";
    test_helper(s, "c10-3.s", "15");
    let s = "(letrec () (begin (pair? (cons '1 '())) '20))";
    test_helper(s, "c10-4.s", "20");
}

#[test]
fn compile11() {
    let s = "(letrec () (vector-set! (make-vector '4) '0 '10))";
    test_helper(s, "c11-1.s", "#<void>");
    let s = "(letrec () (set-car! (cons '1 '2) '10))";
    test_helper(s, "c11-2.s", "#<void>");
    let s = "(letrec () (set-cdr! (cons '1 '2) '14))";
    test_helper(s, "c11-3.s", "#<void>");
}

#[test]
fn compile12() {
    let s = "(if (set-car! (cons '1 '2) '10) '7 '8)";
    test_helper(s, "c12-1.s", "7");
    let s = "(letrec () (if (vector-set! (make-vector '4) '0 '10) '11 '12))";
    test_helper(s, "c12-2.s", "11");
    let s = "'#t";
    test_helper(s, "c12-3.s", "#t");
}

#[test]
fn compile13() {
    let s = "    
    (letrec ([vector-scale!$0 (lambda (vect.1 scale.2)
                                (let ([size.3 (vector-length vect.1)])
                                  (vector-scale!$1 size.3 vect.1 scale.2)))]
             [vector-scale!$1 (lambda (offset.4 vect.5 scale.6)
                                (if (< offset.4 '1)
                                    '0
                                    (begin
                                      (vector-set! vect.5 (- offset.4 '1)
                                        (* (vector-ref vect.5 (- offset.4 '1))
                                           scale.6))
                                      (vector-scale!$1 (- offset.4 '1) vect.5
                                        scale.6))))]
             [vector-sum$2 (lambda (vect.7)
                             (vector-sum$3 (vector-length vect.7) vect.7))]
             [vector-sum$3 (lambda (offset.9 vect.10)
                             (if (< offset.9 '1)
                                 '0
                                 (+ (vector-ref vect.10 (- offset.9 '1))
                                    (vector-sum$3 (- offset.9 '1) vect.10))))])
      (let ([vect.11 (make-vector '5)])
        (begin
          (vector-set! vect.11 '0 '123)
          (vector-set! vect.11 '1 '10)
          (vector-set! vect.11 '2 '7)
          (vector-set! vect.11 '3 '12)
          (vector-set! vect.11 '4 '57)
          (vector-scale!$0 vect.11 '10)
          (vector-sum$2 vect.11))))";
    test_helper(s, "c13.s", "2090");
}

#[test]
fn compile14() {
    let s = "
    (letrec ([div$400 (lambda (d.401 n.402) (div-help$500 d.401 n.402 '0))]
             [div-help$500 (lambda (d.501 n.502 q.503)
                             (if (> n.502 d.501)
                                 q.503
                                 (div-help$500 (- d.501 n.502) n.502 (+ q.503 '1))))])
      (letrec ([alloc$100 (lambda (n.101) (make-vector (div$400 n.101 '8)))]
               [mref$200 (lambda (x.201 y.202)
                           (if (vector? x.201)
                               (vector-ref x.201 (div$400 y.202 '8))
                               (vector-ref y.202 (div$400 x.201 '8))))]
               [mset!$300 (lambda (x.301 y.302 z.303)
                            (begin
                              (if (vector? x.301)
                                  (vector-set! x.301 (div$400 y.302 '8) z.303)
                                  (vector-set! y.302 (div$400 x.301 '8) z.303))
                              (void)))])
        (letrec ([stack-new$0 (lambda (size.1)
                                (let ([store.3 (alloc$100 (* '8 size.1))]
                                      [meths.4 (alloc$100 (* '3 '8))]
                                      [stack.2 (alloc$100 (* '3 '8))])
                                  (begin
                                    (mset!$300 meths.4 '0 stack-push$2)
                                    (mset!$300 meths.4 '8 stack-pop$3)
                                    (mset!$300 meths.4 '16 stack-top$4)
                                    (mset!$300 stack.2 '0 meths.4)
                                    (mset!$300 stack.2 '8 '0)
                                    (mset!$300 stack.2 '16 store.3)
                                    stack.2)))]
                 [invoke$1 (lambda (obj.5 meth-idx.6)
                             (mref$200 (mref$200 obj.5 '0) (* meth-idx.6 '8)))]
                 [stack-push$2 (lambda (self.7 val.8)
                                 (begin
                                   (mset!$300 (mref$200 self.7 '16) 
                                          (* (mref$200 self.7 '8) '8)
                                          val.8)
                                   (mset!$300 self.7 '8 (+ (mref$200 self.7 '8) '1))
                                   self.7))]
                 [stack-pop$3 (lambda (self.9)
                                (begin
                                  (mset!$300 self.9 '8 (- (mref$200 '8 self.9) '1))
                                  (mref$200 (mref$200 self.9 '16) 
                                        (* (mref$200 self.9 '8) '8))))]
                 [stack-top$4 (lambda (self.209)
                                (mref$200 (mref$200 self.209 '16) 
                                      (* (- (mref$200 '8 self.209) '1) '8)))])
          (let ([s1.10 (stack-new$0 '10)])
            (begin
              ((invoke$1 s1.10 '0) s1.10 '10) ;; push '10
              ((invoke$1 s1.10 '0) s1.10 '20) ;; push '20
              ((invoke$1 s1.10 '0) s1.10 '30) ;; push ... well you get the idea
              ((invoke$1 s1.10 '0) s1.10 '40)
              ((invoke$1 s1.10 '0) s1.10 '50)
              ((invoke$1 s1.10 '0) s1.10 '60)
              ((invoke$1 s1.10 '0) s1.10 '70)
              ((invoke$1 s1.10 '0) s1.10 '80)
              ((invoke$1 s1.10 '0) s1.10 '90)
              ((invoke$1 s1.10 '0) s1.10 '100)
              (let ([s2.11 (stack-new$0 '6)])
                (begin
                  ((invoke$1 s2.11 '0) s2.11 ((invoke$1 s1.10 '1) s1.10)) ;; push pop
                  ((invoke$1 s1.10 '1) s1.10) ;; pop
                  ((invoke$1 s2.11 '0) s2.11 ((invoke$1 s1.10 '1) s1.10))
                  ((invoke$1 s1.10 '1) s1.10) ;; pop
                  ((invoke$1 s2.11 '0) s2.11 ((invoke$1 s1.10 '1) s1.10))
                  ((invoke$1 s1.10 '1) s1.10) ;; pop
                  ((invoke$1 s2.11 '0) s2.11 ((invoke$1 s1.10 '1) s1.10))
                  ((invoke$1 s1.10 '1) s1.10) ;; pop
                  ((invoke$1 s2.11 '0) s2.11 ((invoke$1 s1.10 '1) s1.10))
                  ((invoke$1 s2.11 '0) s2.11 ((invoke$1 s1.10 '1) s1.10))
                  (let ([x.1000 (+ ((invoke$1 s2.11 '1) s2.11) ((invoke$1 s2.11 '1) s2.11))])
                    (* (+ (let ([x.1001 (+ ((invoke$1 s2.11 '2) s2.11) ((invoke$1 s2.11 '2) s2.11))])
                            (- x.1001 (+ ((invoke$1 s2.11 '1) s2.11) ((invoke$1 s2.11 '1) s2.11))))
                          (let ([x.1002 (+ ((invoke$1 s2.11 '2) s2.11) ((invoke$1 s2.11 '2) s2.11))])
                            (- (+ ((invoke$1 s2.11 '1) s2.11) ((invoke$1 s2.11 '1) s2.11)) x.1002)))
                       x.1000)))))))))";
    test_helper(s, "c14.s", "0");
}

#[test]
fn compile15() {
    let s = "    
    (let ([v1.13 (make-vector '5)] [p.20 (cons '() (void))])
      (begin
        (vector-set! v1.13 '0 '134)
        (vector-set! v1.13 '1 '123)
        (vector-set! v1.13 '2 '503)
        (vector-set! v1.13 '3 p.20)
        (vector-set! v1.13 '4 '255)
        (let ([v2.14 (make-vector '5)])
          (begin
            (vector-set! v2.14 '0 '134)
            (vector-set! v2.14 '1 '123)
            (vector-set! v2.14 '2 '503)
            (vector-set! v2.14 '3 p.20)
            (vector-set! v2.14 '4 '255)
            (letrec ([vector-equal?$3 (lambda (vect1.8 vect2.9)
                                        (let ([n.15 (vector-length vect1.8)])
                                          (if (= (vector-length vect2.9) n.15)
                                              (vector-equal?$4 vect1.8 vect2.9 (- n.15 '1))
                                              '0)))]
                     [vector-equal?$4 (lambda (vect1.11 vect2.12 off.10)
                                        (if (< off.10 '0)
                                            '#t
                                            (if (eq? (vector-ref vect1.11 off.10)
                                                     (vector-ref vect2.12 off.10))
                                                (vector-equal?$4 vect1.11 vect2.12 (- off.10 '1))
                                                '#f)))])
              (if (eq? (vector-equal?$3 v1.13 v2.14) '#f)
                  '-100
                  (if (eq? (begin
                             (vector-set! v2.14 '3 (cons '() (void)))
                             (vector-equal?$3 v1.13 v2.14))
                           '#f)
                      '200
                      '100)))))))";
    test_helper(s, "c15.s", "200");
}

#[test]
fn compile16() {
    let s = "
    (letrec ([length$3 (lambda (ptr.6)
                         (if (null? ptr.6)
                             '0
                             (+ '1 (length$3 (cdr ptr.6)))))])
      (length$3 (cons '5 (cons '10 (cons '11 (cons '5 (cons '15 '())))))))";
    test_helper(s, "c16.s", "5");
}

#[test]
fn compile17() {
    let s = "
    (let ([x.1 '3])
      (letrec ([f.2 (lambda () x.1)])
        (f.2)))";
    test_helper(s, "c17.s", "3");
}