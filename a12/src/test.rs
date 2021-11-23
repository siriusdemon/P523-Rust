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
fn compile100() {
    let s = "
    (let ([a.1 (letrec ([f.0 (lambda () '80)]) (f.0))]
          [b.2 (letrec ([g.1 (lambda () '50)]) (g.1))])
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
#[should_panic()]
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
#[should_panic()]
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
#[should_panic()]
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
#[should_panic()]
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

#[test]
fn compile18() {
    let s = "    
    (letrec ([mult.2 (lambda (n.4)
                       (letrec ([anon.6 (lambda (m.5) (* n.4 m.5))])
                         anon.6))]
             [succ.1 (lambda (n.3) (+ n.3 '1))])
      (- (* '4 '5) ((mult.2 (succ.1 '3)) (succ.1 '4))))";
    test_helper(s, "c18.s", "0");
}

#[test]
fn compile19() {
    let s = "(let ([x.1 (cons '1 '2)]) (pair? x.1))";
    test_helper(s, "c19.s", "#t");
}

#[test]
fn compile20() {
    let s = "    
    (let ([a.1 (cons '5 '10)])
      (let ([is-pair.2 (pair? a.1)])
        (if is-pair.2 (car a.1) a.1)))";
    test_helper(s, "c20.s", "5");
}

#[test]
fn compile21() {
    let s = "    
    (letrec ([fact.0 (lambda (n.3 k.4)
                       (if (eq? n.3 '0)
                           (k.4 '1)
                           (fact.0 (- n.3 '1)
                                   (letrec ([anon.5 (lambda (v.6)
                                                      (k.4 (* n.3 v.6)))])
                                     anon.5))))]
             [anon.1 (lambda (v.2) v.2)])
      (fact.0 '5 anon.1))";
    test_helper(s, "c21.s", "120");
}

#[test]
fn compile22() {
    let s = "
    (letrec ([even?.1 (lambda (x.3) (if (= x.3 '0) '#t (odd?.2 (- x.3 '1))))]
             [odd?.2 (lambda (x.4) (if (even?.1 x.4) '#f '#t))])
      (cons (even?.1 '17) (odd?.2 '17)))";
    test_helper(s, "c22.s", "(#f . #t)");
}

#[test]
fn compile222() {
    let s = "
    (letrec ([even?.1 (lambda (x.3) (if (= x.3 '0) '#t (odd?.2 (- x.3 '1))))]
             [odd?.2 (lambda (x.4) (if (even?.1 x.4) '#f '#t))])
      (even?.1 '17)";
    test_helper(s, "c222.s", "#f");
}

#[test]
fn compile23() {
    let s = "(begin '7)";
    test_helper(s, "c23-1.s", "7");
    let s = "(letrec () '7)";
    test_helper(s, "c23-2.s", "7");
    let s = "(letrec () (letrec () '7))";
    test_helper(s, "c23-3.s", "7");
    let s = "(let ([x.1 (cons '1 '2)]) (pair? x.1))";
    test_helper(s, "c23-4.s", "#t");
    let s = "
    (let ([x.1 '5] [y.2 '10])
      (begin
        (+ x.1 y.2)
        x.1))";
    test_helper(s, "c23-5.s", "5");
    let s = "
    (let ([tf.1 (cons '#t '#f)])
      (if (car tf.1) '5 '10))";
    test_helper(s, "c23-6.s", "5");
    let s = "
    (let ([tf.1 (cons '#t '#f)])
      (if (cdr tf.1) '5 '10))";
    test_helper(s, "c23-7.s", "10");
    let s = "
    (let ([x.1 '3])
      (letrec ([f.2 (lambda () x.1)])
        (f.2)))";
    test_helper(s, "c23-8.s", "3");
    let s = "
    (cdr (let ([x.1 (cons '1 '2)])
           (begin
             (set-car! x.1 '10)
             (set-cdr! x.1 '20)
             x.1)))";
    test_helper(s, "c23-9.s", "20");
    let s = "
    (let ([x.1 (cons '1 '2)])
      (set-car! x.1 '4))";
    test_helper(s, "c23-10.s", "#<void>");
    let s = "
    (+ (let ([x.1 '3])
         (letrec ([f.2 (lambda (y.3) (+ x.1 y.3))])
           (f.2 '5)))
       '7)";
    test_helper(s, "c23-11.s", "15");
}

#[test]
fn compile24() {
    let s = "(letrec ([f.1 (lambda () f.1)]) (procedure? (f.1)))";
    test_helper(s, "c24-1.s", "#t");
    let s = "
    (letrec ([vectors?.0 (lambda (v.1 v.2)
                           (if (vector? v.1)
                               (vector? v.2)
                               '#f))])
      (let ([v.3 (make-vector '2)] [v.4 (make-vector '2)])
        (begin
          (vector-set! v.3 '0 '10)
          (vector-set! v.3 '1 '20)
          (vector-set! v.4 '0 '5)
          (vector-set! v.4 '1 '15)
          (if (eq? (vectors?.0 v.3 v.4) '#t)
              (+
                (* (vector-ref v.3 '0) (vector-ref v.4 '0))
                (* (vector-ref v.3 '1) (vector-ref v.4 '1)))
              '100))))";
    test_helper(s, "c24-2.s", "350");
    let s = "    
    (let ([x.1 (cons '5 '10)])
      (let ([z.2 (void)])
        (if (set-car! x.1 '5)
            z.2
            (+ '5 '3))))";
    test_helper(s, "c24-3.s", "#<void>");
    let s = "
    (let ([a.1 (cons '5 '10)])
      (let ([is-pair.2 (if (pair? a.1) '#t '#f)])
        (if is-pair.2 (car a.1) a.1)))";
    test_helper(s, "c24-4.s", "5");
    let s = "
    (let ([x.1 '5] [y.2 '7])
      (if (if (= x.1 y.2) (void) (= (+ x.1 '2) y.2)) '172 '63))";
    test_helper(s, "c24-5.s", "172");
}

#[test]
fn compile25() {
    let s = "    
    (if (= (+ '7 (* '2 '4)) (- '20 (+ (+ '1 '1) (+ (+ '1 '1) '1))))
        (+ '1 (+ '1 (+ '1 (+ '1 (+ '1 '10)))))
        '0)";
    test_helper(s, "c25-1.s", "15");
}

#[test]
fn compile26() {
    let s = "
    (letrec ([f.0 (lambda (x.1) (+ '1 x.1))])
      (f.0 (let ([f.2 '3]) (+ f.2 '1))))";
    test_helper(s, "c26-1.s", "5");
    let s = "
    ((letrec ([f.0 (lambda (x.1) (+ '1 x.1))]) f.0)
     (let ([f.2 '3]) (+ f.2 '1)))";
    test_helper(s, "c26-2.s", "5");
    let s = "    
    (cons (letrec ([f.0 (lambda (h.1 v.2) (* h.1 v.2))])
            (letrec ([k.7 (lambda (x.3) (+ x.3 '5))])
              (let ([x.5 '15])
                (letrec ([g.8 (lambda (x.4) (+ '1 x.4))])
                  (k.7 (g.8 (let ([g.6 '3]) (f.0 g.6 x.5))))))))
          '())";
    test_helper(s, "c26-3.s", "(51)");
}