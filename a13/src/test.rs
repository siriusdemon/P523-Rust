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
    ((lambda (y.2)
        ((lambda (f.1) (f.1 (f.1 y.2)))
            (lambda (x.3) (+ x.3 '1))))
     '3)";
    test_helper(s, "c1.s", "5");
}

#[test]
fn compile2() {
    let s = "
    (let ([a.1 (lambda (x.2)
                 (lambda (y.3)
                   (* x.2 (- y.3 '1))))])
      (let ([b.4 (lambda (w.5)
                   ((a.1 '2) '4))])
        (b.4 '6)))";
    test_helper(s, "c2.s", "6");
}

#[test]
fn compile3() {
    let s = "    
    ((lambda (x.1)
       (x.1 ((lambda (y.2) (if y.2 '#f '#t)) '#t)))
      (lambda (z.3) z.3))";
    test_helper(s, "c3.s", "#f");
}

#[test]
fn compile4() {
    let s = "    
    (letrec ([depth.1 (lambda (ls.2)
                        (if (null? ls.2)
                            '1
                            (if (pair? (car ls.2))
                                (let ([l.4 ((lambda (m.6) (+ m.6 '1))
                                            (depth.1 (car ls.2)))]
                                      [r.5 (depth.1 (cdr ls.2))])
                                  (if (< l.4 r.5) r.5 l.4))
                                (depth.1 (cdr ls.2)))))])
      (depth.1
        (cons
          '1
          (cons
            (cons (cons '2 (cons '3 '())) (cons '3 (cons '4 '())))
            (cons '5 '())))))";
    test_helper(s, "c4.s", "3");
}

#[test]
fn compile5() {
    let s = "    
    (let ([f.1 (lambda () (void))]
          [x.2 '5]
          [f.3 (lambda () '10)])
      (eq? (f.1) x.2))))";
    test_helper(s, "c5.s", "#f");
}

#[test]
fn compile6() {
    let s = "    
    (let ([quote.3720 (lambda (x.3715) x.3715)]
          [let.3719 (lambda (x.3714 y.3713) (- y.3713 x.3714))]
          [if.3718 (lambda (x.3712 y.3711 z.3710)
                     (cons x.3712 z.3710))]
          [cons.3717 (lambda (x.3709 y.3708) (cons y.3708 x.3709))]
          [|+.3721| '16])
      (let ([|+.3716| (cons |+.3721| (void))])
        (begin
          (set-car! |+.3716| (* '16 '2))
          (cons.3717
            (let.3719 ((quote.3720 (lambda () '0))) (car |+.3716|))
            (if.3718 (quote.3720 (if '#f '#f '#t)) '720000 '-1)))))";
    test_helper(s, "c6.s", "((#t . -1) . 32)");
}

#[test]
fn compile7() {
    let s = "    
    (letrec ([fib.3722 (lambda (x.3726)
                         (let ([x.3723 (cons x.3726 (void))])
                           (let ([decrx.3725 (lambda ()
                                               (lambda (i.3724)
                                                 (set-car!
                                                   x.3723
                                                   (- (car x.3723)
                                                      i.3724))))])
                             (if (< (car x.3723) '2)
                               '1
                               (+ (begin
                                    ((decrx.3725) '1)
                                    (fib.3722 (car x.3723)))
                                  (begin
                                    ((decrx.3725) '1)
                                    (fib.3722 (car x.3723))))))))])
      (fib.3722 '10))";
    test_helper(s, "c7.s", "384000");
}

#[test]
fn compile8() {
    let s = "    
    (letrec ([curry-list.3752 (lambda (x.3755)
                                (lambda (y.3756)
                                  (lambda (z.3757)
                                    (lambda (w.3758)
                                      (cons
                                        x.3755
                                        (cons
                                          y.3756
                                          (cons
                                            z.3757
                                            (cons w.3758 '()))))))))]
             [append.3751 (lambda (ls1.3754 ls2.3753)
                            (if (null? ls1.3754)
                              ls2.3753
                              (cons
                                (car ls1.3754)
                                (append.3751 (cdr ls1.3754) ls2.3753))))])
      (append.3751
        ((((curry-list.3752 '1) '2) '3) '4)
        ((((curry-list.3752 '5) '6) '7) '8)))";
    test_helper(s, "c8.s", "(1 2 3 4 5 6 7 . 8)");
}

#[test]
fn compile9() {
    let s = "    
    ((((((lambda (x.1)
           (lambda (y.2)
             (lambda (z.3)
               (lambda (w.4)
                 (lambda (u.5)
                   (+ x.1 (+ y.2 (+ z.3 (+ w.4 u.5)))))))))
         '5)
        '6)
       '7)
      '8)
     '9)";
    test_helper(s, "c9.s", "35");
}

#[test]
fn compile99() {
    let s = "     
    ((((lambda (x.1)
           (lambda (y.2)
             (lambda (z.3)
                   (+ x.1 (+ y.2 (+ z.3 '1))))))
       '7)
      '8)
     '9)";
    test_helper(s, "c99.s", "25");
}


#[test]
fn compile10() {
    let s = "    
    (let ([double.3966 (lambda (a.3965) (+ a.3965 a.3965))])
      (double.3966 '10))";
    test_helper(s, "c10.s", "20");
}

#[test]
fn compile11() {
    let s = "    
    ((lambda (y.3967)
       ((lambda (f.3969) (f.3969 (f.3969 y.3967)))
        (lambda (y.3968) y.3968)))
     '4)";
    test_helper(s, "c11.s", "4");
}

#[test]
fn compile12() {
    let s = "    
    (letrec ([thunk-num.3978 (lambda (n.3984)
                               (lambda () n.3984))]
             [force.3977 (lambda (th.3983) (th.3983))]
             [add-ths.3976 (lambda (th1.3982 th2.3981 th3.3980 th4.3979)
                             (+ (+ (force.3977 th1.3982)
                                   (force.3977 th2.3981))
                                (+ (force.3977 th3.3980)
                                   (force.3977 th4.3979))))])
      (add-ths.3976
        (thunk-num.3978 '5)
        (thunk-num.3978 '17)
        (thunk-num.3978 '7)
        (thunk-num.3978 '9)))";
    test_helper(s, "c12.s", "38");
}

#[test]
fn compile13() {
    let s = "    
    (letrec ([count-leaves.4020 (lambda (p.4021)
                                  (if (pair? p.4021)
                                    (+ (count-leaves.4020 (car p.4021))
                                       (count-leaves.4020 (cdr p.4021)))
                                    '1))])
      (count-leaves.4020
        (cons
          (cons '0 (cons '0 '0))
          (cons
            (cons (cons (cons '0 (cons '0 '0)) '0) '0)
            (cons
              (cons (cons '0 '0) (cons '0 (cons '0 '0)))
              (cons (cons '0 '0) '0))))))";
    test_helper(s, "c13.s", "16");
}

#[test]
fn compile14() {
    let s = "    
    (let ([t.4024 (cons
                    '5
                    (cons '10 (cons '11 (cons '5 (cons '15 '())))))])
      ((letrec ([length.4022 (lambda (ptr.4023)
                               (if (null? ptr.4023)
                                 '0
                                 (+ '1 (length.4022 (cdr ptr.4023)))))])
         length.4022)
       t.4024))";
    test_helper(s, "c14.s", "5");
}

#[test]
fn compile15() {
    let s = "    
    (let ([t.13 (let ([tmp.14 (make-vector '5)])
                  (begin
                    (vector-set! tmp.14 '0 '123)
                    (vector-set! tmp.14 '1 '10)
                    (vector-set! tmp.14 '2 '7)
                    (vector-set! tmp.14 '3 '12)
                    (vector-set! tmp.14 '4 '57)
                    tmp.14))])
      (let ([vect.1 t.13])
        (begin
          (letrec ([vector-scale!.2 (lambda (vect.4 scale.3)
                                      (let ([size.5 (vector-length vect.4)])
                                        (letrec ([f.6 (lambda (idx.7)
                                                        (if (>= idx.7 '1)
                                                          (let ([idx.8 (- idx.7
                                                                          '1)])
                                                            (begin
                                                              (vector-set!
                                                                vect.4
                                                                idx.8
                                                                (* (vector-ref
                                                                     vect.4
                                                                     idx.8)
                                                                   scale.3))
                                                              (f.6 idx.8)))
                                                          (void)))])
                                          (f.6 size.5))))])
            (vector-scale!.2 vect.1 '10))
          (letrec ([vector-sum.9 (lambda (vect.10)
                                   (letrec ([f.11 (lambda (idx.12)
                                                    (if (< idx.12 '1)
                                                      '0
                                                      (+ (vector-ref
                                                           vect.10
                                                           (- idx.12 '1))
                                                         (f.11
                                                           (- idx.12
                                                              '1)))))])
                                     (f.11 (vector-length vect.10))))])
            (vector-sum.9 vect.1)))))";
    test_helper(s, "c15.s", "2090");
}

#[test]
fn compile16() {
    let s = "(letrec () (begin (> '7 '8) '8))";
    test_helper(s, "c16.s", "8");
}

#[test]
fn compile17() {
    let s = "    
    (letrec ([f.3871 (lambda (x.3877) (+ '1 x.3877))]
             [g.3870 (lambda (x.3876) (- x.3876 '1))]
             [t.3869 (lambda (x.3875) (- x.3875 '1))]
             [j.3868 (lambda (x.3874) (- x.3874 '1))]
             [i.3867 (lambda (x.3873) (- x.3873 '1))]
             [h.3866 (lambda (x.3872) (- x.3872 '1))])
      (let ([x.3878 '80])
        (let ([a.3881 (f.3871 x.3878)]
              [b.3880 (g.3870 x.3878)]
              [c.3879 (h.3866 (i.3867 (j.3868 (t.3869 x.3878))))])
          (* a.3881 (* b.3880 (+ c.3879 '0))))))";
    test_helper(s, "c17.s", "486324");
}

#[test]
fn compile18() {
    let s = "    
    (letrec ([fold.0 (lambda (proc.1 base.2 ls.3)
                       (if (null? ls.3)
                           base.2
                           (proc.1 (car ls.3)
                                   (fold.0 proc.1 base.2 (cdr ls.3)))))])
      (fold.0 (lambda (x.4 y.5) (* x.4 y.5))
              '1
              (cons '1 (cons '2 (cons '3 (cons '4 (cons '5 '())))))))";
    test_helper(s, "c18.s", "120");
}

#[test]
fn compile19() {
    let s = "    
    (let ([fill.5 (lambda (x.1 v.2)
                    (if (vector? v.2)
                        (let ([length.4 (vector-length v.2)])
                          (if (fixnum? x.1)
                              (if (<= x.1 length.4)
                                  (letrec ([loop.6 (lambda (index.3)
                                                     (if (= index.3 length.4)
                                                         '#t
                                                         (begin
                                                           (vector-set!
                                                             v.2
                                                             index.3
                                                             x.1)
                                                           (loop.6
                                                             (+ index.3
                                                                '1)))))])
                                    (loop.6 '0))
                                  '#f)
                              '#f))
                        '#f))])
      (fill.5 '3 (make-vector '10)))";
    test_helper(s, "c19.s", "#t");
}