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

#[test]
fn compile12() {
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
    test_helper(s, "c12.s", "2090");
}

#[test]
fn compile13() {
    let s = "
    (letrec ()
      (let ([a (make-vector '10)])
        (begin
          (vector-length a)))";
    test_helper(s, "c13.s", "10");
}

#[test]
fn compile14() {
    let s = "
    (letrec ([count-leaves$3 (lambda (ptr.6)
                               (if (pair? ptr.6)
                                   (+ (count-leaves$3 (car ptr.6))
                                      (count-leaves$3 (cdr ptr.6)))
                                   '1))])
      (count-leaves$3
        (cons 
          (cons
            '0
            (cons '0 '0))
          (cons
            (cons
              (cons (cons '0 (cons '0 '0)) '0)
              '0)
            (cons (cons (cons '0 '0) (cons '0 (cons '0 '0)))
                  (cons (cons '0 '0) '0))))))";
    test_helper(s, "c14.s", "16");
}

#[test]
fn compile15() {
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
    test_helper(s, "c15.s", "65");
}

#[test]
fn compile16() {
    let s = "
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
                            (void)))]
             [div$400 (lambda (d.401 n.402) (div-help$500 d.401 n.402 '0))]
             [div-help$500 (lambda (d.501 n.502 q.503)
                             (if (> n.502 d.501)
                                 q.503
                                 (div-help$500 (- d.501 n.502) n.502 (+ q.503 '1))))]
             [stack-new$0 (lambda (size.1)
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
                   x.1000)))))))";
    test_helper(s, "c16.s", "0");
}

#[test]
fn compile17() {
    let s = "
    (letrec ([f$1 (lambda (x.1) x.1)])
      (let ([v (make-vector '2)])
        (begin 
          (vector-set! v '0 f$1)
          (vector-set! v '1 '1)
          ((vector-ref v '0) (vector-ref v '1)))))";
    test_helper(s, "c17.s", "1");
}