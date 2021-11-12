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
    assert_eq!(r.as_str(), expect);
}

#[test]
fn compile1() {
    let s = "
    (letrec ([main$0 (lambda (a.1 b.2 d.5)
                       (locals (c.3)
                         (begin
                           (set! c.3 
                             (if (if (= a.1 1) (true) (= b.2 1))
                                 1
                                 0))
                           (+ c.3 5))))])
      (locals () (main$0 0 1 2)))";
    test_helper(s, "c1.s", "6\n");
}

#[test]
fn compile2() {
    let s = " 
    (letrec ()
      (locals (a.1)
        (begin
          (set! a.1 10)
          (if (< 7 a.1)
              (nop)
              (set! a.1 (+ a.1 a.1)))
          a.1)))";
    test_helper(s, "c2.s", "10\n");
}

#[test]
fn compile3() {
    let s = "
    (letrec ()
      (locals (n.1 a.2 b.3 c.4)
        (begin
          (set! n.1 1)
          (begin
            (set! a.2 2)
            (begin
              (set! b.3 3)
              (set! n.1 (+ n.1 (if (= (+ n.1 b.3) b.3) 5 10)))
              (set! n.1 (+ n.1 b.3)))
            (set! n.1 (+ n.1 a.2)))
          (+ n.1 n.1))))";
    test_helper(s, "c3.s", "32\n");
}

#[test]
fn compile4() {
    let s = "
    (letrec ()
       (locals (x.1 y.2 result.3)
         (begin
           (set! result.3 (+ (if (begin 
                                   (set! x.1 5) 
                                   (set! y.2 10)
                                   (< 11 x.1))
                                 (+ x.1 y.2)
                                 (+ y.2 100))
                             (begin
                               (set! x.1 10)
                               (set! y.2 20)
                               (* x.1 y.2))))
           result.3)))";
    test_helper(s, "c4.s", "310\n");
}

#[test]
fn compile5() {
    let s = "
    (letrec ([div$0 (lambda (x.1)
                       (locals ()
                         (begin 
                           (set! x.1 (sra x.1 1)) 
                           (div$1 x.1))))]
              [div$1 (lambda (result.1)
                       (locals () result.1))])
       (locals (label-temp.1)
         (begin
           (set! label-temp.1 div$0)
           (label-temp.1 64))))";
    test_helper(s, "c5.s", "32\n");
}

// I leave this test here to remind myself that the register allocator is not optimized.
#[test]
fn compile6() {
    let s = "
    (letrec ()
       (locals (b.2 g.7 c.3 d.4 e.5 a.1 f.6)
         (begin
           (set! a.1 1)
           (set! b.2 2)
           (set! c.3 a.1)
           (set! d.4 4)
           (set! e.5 5)
           (set! f.6 b.2)
           (set! f.6 (+ f.6 c.3))
           (set! f.6 (+ f.6 d.4))
           (set! f.6 (+ f.6 e.5))
           (set! g.7 7)
           (+ f.6 g.7))))";
    test_helper(s, "c6.s", "19\n");
}

#[test]
fn compile7() {
    let s = "
    (letrec ([sum$1 (lambda (x.1 y.2 z.3 w.4)
                      (locals ()
                        (+ x.1 (+ y.2 (+ z.3 w.4)))))])
      (locals (a.1)
        (sum$1 (begin (set! a.1 1) a.1)
               (begin (set! a.1 2) a.1)
               (begin (set! a.1 3) a.1)
               (begin (set! a.1 4) a.1))))";
    test_helper(s, "c7.s", "10\n");
}

#[test]
fn compile8() {
    let s = "
    (letrec ()
      (locals (a.1 b.2 c.3 d.4 e.5 f.6 g.7 h.8 i.9 j.10 k.11 l.12 m.13 n.14 
               o.15 p.16 q.17 r.18 s.19 t.20 u.21 v.22 w.23 x.24 y.25 z.26)
        (begin
          (set! a.1 1)
          (set! b.2 2)
          (set! c.3 3)
          (set! d.4 4)
          (set! e.5 5)
          (set! f.6 6)
          (set! g.7 7)
          (set! h.8 8)
          (set! i.9 9)
          (set! j.10 10)
          (set! k.11 11)
          (set! l.12 12)
          (set! m.13 13)
          (set! n.14 14)
          (set! o.15 15)
          (set! p.16 16)
          (set! q.17 17)
          (set! r.18 18)
          (set! s.19 19)
          (set! t.20 20)
          (set! u.21 21)
          (set! v.22 22)
          (set! w.23 23)
          (set! x.24 24)
          (set! y.25 25)
          (set! z.26 26)
          (set! a.1 (+ a.1 (+ b.2 (+ c.3 (+ d.4 (+ e.5 (+ f.6 (+ g.7 (+ h.8
                    (+ i.9 (+ j.10 (+ k.11 (+ l.12 (+ m.13 (+ n.14 (+ o.15 
                    (+ p.16 (+ q.17 (+ r.18 (+ s.19 (+ t.20 (+ u.21 (+ v.22 
                    (+ w.23 (+ x.24 (+ y.25 z.26))))))))))))))))))))))))))
          (set! b.2 27)
          (set! c.3 28)
          (set! d.4 29)
          (set! e.5 30)
          (set! f.6 31)
          (set! g.7 32)
          (set! h.8 33)
          (set! i.9 34)
          (set! j.10 35)
          (set! k.11 36)
          (set! l.12 37)
          (set! m.13 38)
          (set! n.14 39)
          (set! o.15 40)
          (set! a.1 (+ a.1 (+ b.2 (+ c.3 (+ d.4 (+ e.5 (+ f.6 (+ g.7 (+ h.8
                    (+ i.9 (+ j.10 (+ k.11 (+ l.12 (+ m.13 
                    (+ n.14 o.15)))))))))))))))
          a.1)))))";
    test_helper(s, "c8.s", "820\n");
}

#[test]
fn compile9() {
    let s = "
    (letrec ([fact$0 (lambda (n.1)
                       (locals ()
                         (fact$1 n.1 1)))]
             [fact$1 (lambda (n.1 a.2)
                       (locals ()
                         (if (= n.1 0)
                             a.2
                             (fact$1 (- n.1 1) (* n.1 a.2)))))])
      (locals () (fact$0 10)))";
    test_helper(s, "c9.s", "3628800\n");
}

#[test]
fn compile10() {
    let s = "
    (letrec ([if-test$1 (lambda ()
                           (locals (x.5)
                             (* (if (begin (set! x.5 5) (= x.5 5))
                                    (+ x.5 10)
                                    (- x.5 10)) 10)))])
       (locals () (if-test$1)))";
    test_helper(s, "c10.s", "150\n");
}

// remind myself that the (value value*) is not supported yet.
#[test]
#[should_panic()]
fn compile11() {
    let s = "
    (letrec ([f1 (lambda () (locals () 42))]
             [f2 (lambda () (locals () 10))])
            
        (locals (x.1)
            (set! x.1 1)
            ((if (= x.1 1) f1 f2))))";
    test_helper(s, "c11.s", "42\n");
}

#[test]
fn compile12() {
    let s = "
    (letrec ([f$0 (lambda (h.1 v.2) (locals () (* h.1 v.2)))]
             [k$1 (lambda (x.1) (locals () (+ x.1 5)))]
             [g$2 (lambda (x.1) (locals () (+ 1 x.1)))])
      (locals (x.4 g.1)
        (begin
          (set! x.4 15)
          (k$1 (g$2 (begin (set! g.1 3) (f$0 g.1 x.4)))))))";
    test_helper(s, "c12.s", "51\n");
}

#[test]
fn compile13() {
    let s = "
    (letrec ([one$1 (lambda (n.1) 
                      (locals () (if (= 0 n.1) 1 (one$1 (- n.1 1)))))])
       (locals () (one$1 13)))";
    test_helper(s, "c13.s", "1\n");
}


#[test]
fn compile14() {
    let s = "(letrec () (locals () 7))";
    test_helper(s, "c14-1.s", "7\n");
    let s = "(letrec () (locals () (+ 5 7)))";
    test_helper(s, "c14-2.s", "12\n");
    let s = "(letrec () (locals () (+ 7 (* 5 7))))";
    test_helper(s, "c14-3.s", "42\n");
    let s = "(letrec () (locals () (* (+ 2 4) (+ (+ 6 7) 4))))";
    test_helper(s, "c14-4.s", "102\n");
    let s = "
    (letrec () 
      (locals () 
        (if (= (+ 7 (* 2 4)) (- 20 (+ (+ 1 1) (+ (+ 1 1) 1))))
            (+ 1 (+ 1 (+ 1 (+ 1 (+ 1 10)))))
            0)))";
    test_helper(s, "c14-5.s", "15\n");
}

#[test]
fn compile15() {
    let s = "
    (letrec ()
      (locals (a.1)
        (begin
          (set! a.1 5)
          (if (< 5 a.1)
              a.1
              (+ a.1 a.1)))))";
    test_helper(s, "c15.s", "10\n");
}

#[test]
fn compile16() {
    let s = "
    (letrec ()
      (locals (c.1 a.2)
        (begin
          (set! a.2 5)
          (set! c.1 10)
          (if (< a.2 c.1) a.2 c.1))))";
    test_helper(s, "c16.s", "5\n");
}

#[test]
fn compile17() {
    let s = "
     (letrec ()
       (locals (a.1)
         (begin
           (set! a.1 5)
           (if (< a.1 10) (set! a.1 (* a.1 10)) (nop))
           a.1)))";
    test_helper(s, "c17.s", "50\n");
}

#[test]
fn compile18() {
    let s = "
    (letrec ([f$0 (lambda (x.1) (locals () (+ 1 x.1)))])
      (locals (f.1) (f$0 (begin (set! f.1 3) (+ f.1 1)))))";
    test_helper(s, "c18.s", "5\n");
}

#[test]
fn compile19() {
    let s = "
    (letrec ([a$0 (lambda (u.1 v.2 w.3 x.4) 
                    (locals () 
                      (if (= u.1 0) 
                          (b$1 v.2 w.3 x.4)
                          (a$0 (- u.1 1) v.2 w.3 x.4))))]
             [b$1 (lambda (q.1 r.2 x.4)
                    (locals (p.3)
                      (begin
                        (set! p.3 (* q.1 r.2))
                        (e$3 (* q.1 r.2) p.3 x.4))))]
             [c$2 (lambda (x.1) (locals () (* 5 x.1)))]
             [e$3 (lambda (n.1 p.3 x.4)
                    (locals ()
                      (if (= n.1 0) 
                          (c$2 p.3)
                          (o$4 (- n.1 1) p.3 x.4))))]
             [o$4 (lambda (n.1 p.3 x.4) 
                    (locals ()
                      (if (= 0 n.1)
                          (c$2 x.4)
                          (e$3 (- n.1 1) p.3 x.4))))])
      (locals (x.4)
        (begin
          (set! x.4 5)
          (a$0 3 2 1 x.4))))";
    test_helper(s, "c19.s", "10\n");
}

#[test]
fn compile20() {
    let s = "
     (letrec ([f$0 (lambda () (locals () 80))])
      (locals (a.1 b.2)
        (begin
          (set! a.1 (f$0))
          (set! b.2 (f$0))
          (* a.1 b.2))))";
    test_helper(s, "c20.s", "6400\n");
}

#[test]
fn compile21() {
    let s = "
     (letrec ([f$0 (lambda () (locals () 80))]
             [g$1 (lambda () (locals () 50))])
      (locals (a.1 b.2)
        (begin
          (set! a.1 (f$0))
          (set! b.2 (g$1))
          (* a.1 b.2))))";
    test_helper(s, "c21.s", "4000\n");

}

#[test]
fn compile22() {
    let s = "
    (letrec ([f$0 (lambda (x.1) (locals () (+ x.1 1)))]
             [g$1 (lambda (y.2) (locals () (f$0 (f$0 y.2))))])
      (locals () (+ (f$0 1) (g$1 1)))) ";
    test_helper(s, "c22.s", "5\n");
}

#[test]
fn compile23() {
    let s = "
     (letrec ([fact$0 (lambda (n.1) 
                       (locals () 
                         (if (= n.1 0) 1 (* n.1 (fact$0 (- n.1 1))))))])
      (locals () (fact$0 10))) ";
    test_helper(s, "c23.s", "3628800\n");
}

#[test]
fn compile24() {
    let s = "
     (letrec ([double$0 (lambda (a.1)
                          (locals () (+ a.1 a.1)))])
       (locals () (double$0 10))) ";
    test_helper(s, "c24.s", "20\n");
}

#[test]
fn compile25() {
    let s = "
    (letrec ([double$1 (lambda (x.1)
                          (locals ()
                            (* x.1 2)))])
       (locals () (begin (double$1 5))))";
    test_helper(s, "c25.s", "10\n");
}

#[test]
fn compile26() {
    let s = "
      (letrec ()
       (locals (x.5 y.10)
         (begin 
           (set! x.5 (begin (set! y.10 10) (set! x.5 15) (* y.10 x.5)))
           x.5)))";
    test_helper(s, "c26.s", "150\n");
}

#[test]
fn compile27() {
    let s = "
    (letrec ([f$0 (lambda (x.1) (locals () (+ 1 x.1)))]
             [g$1 (lambda (x.1) (locals () (- x.1 1)))]
             [t$2 (lambda (x.1) (locals () (- x.1 1)))]
             [j$3 (lambda (x.1) (locals () (- x.1 1)))]
             [i$4 (lambda (x.1) (locals () (- x.1 1)))]
             [h$5 (lambda (x.1) (locals () (- x.1 1)))])
      (locals (x.1 a.2 b.3 c.4)
        (begin
          (set! x.1 80)
          (set! a.2 (f$0 x.1))
          (set! b.3 (g$1 x.1))
          (set! c.4 (h$5 (i$4 (j$3 (t$2 x.1)))))
          (* a.2 (* b.3 (+ c.4 0))))))";
    test_helper(s, "c27.s", "486324\n");
}

#[test]
fn compile28() {
    let s = "
    (letrec ([fact$0 (lambda (n.1)
                       (locals (t.2 t.3)
                         (if (= n.1 0)
                             1
                             (begin
                               (set! t.2 (- n.1 1))
                               (set! t.3 (fact$0 t.2))
                               (* n.1 t.3)))))])
      (locals () (fact$0 10)))";
    test_helper(s, "c28.s", "3628800\n");
}

#[test]
fn compile29() {
    let s = "
    (letrec ([fib$0 (lambda (n.1)
                      (locals ()
                        (if (if (= 0 n.1) (true) (= 1 n.1))
                            1
                            (+ (fib$0 (- n.1 1)) (fib$0 (- n.1 2))))))])
      (locals () (fib$0 10)))";
    test_helper(s, "c29.s", "89\n");
}

#[test]
fn compile30() {
    let s = "
    (letrec ([even$0 (lambda (n.1)
                       (locals ()
                         (if (= n.1 0)
                             1
                             (odd$1 (- n.1 1)))))]
             [odd$1 (lambda (n.1)
                      (locals ()
                        (if (= n.1 0)
                            0
                            (even$0 (- n.1 1)))))])
      (locals () (even$0 17)))";
    test_helper(s, "c30.s", "0\n");
}
#[test]
fn compile31() {
    let s = "
     (letrec ()
       (locals (x.5) 
         (begin (set! x.5 5) x.5)))";
    test_helper(s, "c31.s", "5\n");
}

#[test]
fn compile32() {
    let s = "
     (letrec ()
       (locals (x.5 y.6)
         (begin
           (set! x.5 5)
           (set! y.6 6)
           (+ x.5 y.6))))";
    test_helper(s, "c32.s", "11\n");
}

#[test]
fn compile33() {
    let s = "
     (letrec ([expt$0 (lambda (n.1 m.2)
                        (locals ()
                          (if (= m.2 1)
                              n.1
                              (* n.1 (expt$0 n.1 (- m.2 1))))))]
              [div$1 (lambda (n.1 d.2)
                       (locals ()
                         (div-helper$2 31 (- (* 2 n.1) d.2) 
                                       (* d.2 (expt$0 2 32)) 0)))]
              [div-helper$2 (lambda (i.1 p.2 d.3 q.4)
                              (locals ()
                                (if (> 0 i.1)
                                    q.4
                                    (if (>= p.2 0)
                                        (div-helper$2 (- i.1 1)
                                                      (- (* 2 p.2) d.3)
                                                      d.3
                                                      (logor (expt$0 2 i.1)
                                                             q.4))
                                        (div-helper$2 (- i.1 1)
                                                      (- (* 2 (+ p.2 d.3)) d.3)
                                                      d.3
                                                      q.4)))))])
       (locals () (div$1 153 17)))";
    // Yes, I am very serious to fill the answoer here.
    // If you remove every locals and one pair of parentheses in the program above 
    // and run it with chez scheme, it really output 2147483656. Try with test33.scm
    // So I think it is the right answer for this test.
    test_helper(s, "c33.s", "2147483656\n");
}

#[test]
fn compile34() {
    let s = "
     (letrec ([setbit3$0 (lambda (x.1)
                           (locals ()
                             (begin
                               (set! x.1 (logor x.1 8))
                               (return$1 x.1))))]
              [return$1 (lambda (x.1)
                          (locals ()
                            (begin x.1)))])
       (locals ()
         (begin (setbit3$0 1))))";
    test_helper(s, "c34.s", "9\n");

}

#[test]
fn compile35() {
    let s = "
    (letrec ([zero?$0 (lambda (n.1)
                         (locals (x.5)
                           (begin
                             (set! x.5 0)
                             (set! x.5 (- x.5 n.1))
                             (set! x.5 (sra x.5 63))
                             (set! x.5 (logand x.5 1))
                             (return$1 x.5))))]
              [return$1 (lambda (x.5)
                          (locals () x.5))])
       (locals () (zero?$0 5)))";
    test_helper(s, "c35.s", "1\n");
}

#[test]
fn compile36() {
    let s = "
     (letrec ([sqr-double$0 (lambda (z.5)
                              (locals ()
                                (begin
                                  (set! z.5 (* z.5 z.5))
                                  (double$1 z.5))))]
              [double$1 (lambda (w.4)
                          (locals ()
                            (begin
                              (set! w.4 (+ w.4 w.4))
                              (return$3 w.4))))]
              [return$3 (lambda (result.1)
                          (locals () result.1))])
       (locals () (begin (sqr-double$0 3) (sqr-double$0 5))))";
    test_helper(s, "c36.s", "50\n");
}

#[test]
fn compile37() {
    let s = "
      (letrec ([square$1 (lambda (x.1)
                          (locals ()
                            (begin (* x.1 x.1))))])
       (locals () (square$1 7)))";
    test_helper(s, "c37.s", "49\n");
}


// I have not valiate its value, just ensure it output something SEEMS correct.
#[test]
fn compile38() {
    let s = "
    (letrec ([f$1 (lambda (n.2 a.3 b.4 c.5 x.6)
                    (locals ()
                      (if (= n.2 0)
                          (+ (* a.3 (* x.6 x.6)) (+ (* b.4 x.6) c.5))
                          (+ (f$1 (sra n.2 3)
                                  (+ a.3 (logand n.2 4))
                                  (+ b.4 (logand n.2 2))
                                  (+ c.5 (logand n.2 1))
                                  x.6)
                             1))))])
      (locals () (f$1 16434824 1 0 -1 7)))";
    test_helper(s, "c38.s", "900\n");
}

// I have not valiate its value, just ensure it output something SEEMS correct.
#[test]
fn compile39() {
    let s = "
    (letrec ([f$1 (lambda (n.2 a.3 b.4 c.5 x.6)
                    (locals ()
                      (if (= n.2 0)
                          (+ (* a.3 (* x.6 x.6)) (+ (* b.4 x.6) c.5))
                          (- (f$1 (sra n.2 3)
                                  (+ a.3 (logand n.2 4))
                                  (+ b.4 (logand n.2 2))
                                  (+ c.5 (logand n.2 1))
                                  x.6)
                             (g$0 n.2 a.3 b.4 c.5)))))]
             [g$0 (lambda (n.7 a.8 b.9 c.10)
                    (locals () (+ (- n.7 a.8) (- b.9 c.10))))])
      (locals () (f$1 16434824 1 0 -1 7)))";
    test_helper(s, "c39.s", "-18781744\n");
}

#[test]
fn compile40() {
    let s = "
    (letrec ([square$0 (lambda (n.1) (locals () (* n.1 n.1)))])
      (locals () (square$0 10)))";
    test_helper(s, "c40.s", "100\n");
}

#[test]
fn compile41() {
    let s = "
     (letrec ([gcd$0 (lambda (x.1 y.2)
                       (locals ()
                        (if (= y.2 0) 
                            x.1 
                            (gcd$0 (if (> x.1 y.2) (- x.1 y.2) x.1)
                                   (if (> x.1 y.2) y.2 (- y.2 x.1))))))])
      (locals () (gcd$0 1071 1029)))";
    test_helper(s, "c41.s", "21\n");
}

#[test]
fn compile42() {
    let s = "
     (letrec ([ack$0 (lambda (m.1 n.2)
                      (locals (tmp.3)
                        (if (= m.1 0)
                            (+ n.2 1)
                            (if (if (> m.1 0) (= n.2 0) (false))
                                (ack$0 (- m.1 1) 1)
                                (begin
                                  (set! tmp.3 (ack$0 m.1 (- n.2 1)))
                                  (ack$0 (- m.1 1) tmp.3))))))])
      (locals () (ack$0 2 4))) ";
    test_helper(s, "c42.s", "11\n");
}

#[test]
fn compile43() {
    let s = "
    (letrec ([ack$0 (lambda (m.1 n.2)
                      (locals ()
                        (if (= m.1 0)
                            (+ n.2 1)
                            (if (if (> m.1 0) (= n.2 0) (false))
                                (ack$0 (- m.1 1) 1)
                                (ack$0 (- m.1 1) (ack$0 m.1 (- n.2 1)))))))])
      (locals () (ack$0 2 4))) ";
    test_helper(s, "c43.s", "11\n");
}

#[test]
fn compile44() {
    let s = "
    (letrec ([fib$0 (lambda (n.1) (locals () (fib$1 n.1 0 1)))]
             [fib$1 (lambda (n.1 a.2 b.3)
                      (locals ()
                        (if (= n.1 0)
                            a.2
                            (fib$1 (- n.1 1) b.3 (+ b.3 a.2)))))])
      (locals () (fib$0 5)))";
    test_helper(s, "c44.s", "5\n");

}

#[test]
fn compile45() {
    let s = "
    (letrec ([if-test$2 (lambda ()
                           (locals (x.5)
                             (begin
                               (set! x.5 (if (begin
                                               (set! x.5 7)
                                               (if (< x.5 1)
                                                   (false)
                                                   (< x.5 10)))
                                           (* x.5 2)
                                           (+ x.5 5)))
                               x.5)))])
       (locals () (if-test$2)))";
    test_helper(s, "c45.s", "14\n");
}

fn compile46() {
    let s = "
    (letrec ([if-test$3 (lambda (n.1)
                           (locals ()
                             (begin
                               (if (if (= n.1 0)
                                       (true)
                                       (if (= n.1 1) (true) (= n.1 2)))
                                   (* n.1 5)
                                   (- n.1 5)))))])
       (locals () (if-test$3 2)))";
    test_helper(s, "c47.s", "10\n");
}

#[test]
fn compile48() {
    let s = "
    (letrec ([if-test$4 (lambda (x.5)
                           (locals ()
                             (begin
                               (* (if (if (= x.5 10) (false) (true))
                                      (+ x.5 10)
                                      (- x.5 2))
                                  10))))])
      (locals () (if-test$4 2)))";
    test_helper(s, "c48.s", "120\n");
}

#[test]
fn compile49() {
    let s = "
    (letrec ([if-test$5 (lambda (n.1 x.2 y.3)
                           (locals ()
                             (begin
                               (if (= n.1 0)
                                   (set! x.2 (+ x.2 y.3))
                                   (set! y.3 (+ y.3 x.2)))
                               (set! x.2 (+ x.2 n.1))
                               (if (if (= n.1 y.3) (false) (true))
                                   (+ n.1 x.2)
                                   (+ n.1 y.3)))))])
       (locals () (begin (if-test$5 1 1 1))))";
    test_helper(s, "c49.s", "3\n");
}

#[test]
fn compile50() {
    let s = "
    (letrec ([if-test$6 (lambda (n.1)
                           (locals (x.2 y.3)
                             (begin
                               (set! x.2 1)
                               (begin
                                 (set! y.3 1)
                                 (if (= n.1 0)
                                     (set! x.2 (+ x.2 y.3))
                                     (set! y.3 (+ y.3 x.2)))
                                 (set! x.2 (+ x.2 n.1)))
                               (if (if (= n.1 y.3) (false) (true))
                                   (set! n.1 (+ n.1 x.2))
                                   (set! n.1 (+ n.1 y.3)))
                               (+ x.2 n.1))))])
       (locals ()(if-test$6 1)))";
    test_helper(s, "c50.s", "5\n");
}

#[test]
fn compile51() {
    let s = "
    (letrec ()
       (locals (x.1 y.2 z.3)
         (begin
           (set! x.1 0)
           (set! y.2 1)
           (if (if (= x.1 0) (= y.2 1) (false))
               (set! z.3 5)
               (begin (set! z.3 5) (set! z.3 (+ z.3 z.3))))
           z.3)))";
    test_helper(s, "c51.s", "5\n");
}

#[test]
fn compile52() {
    let s = "
    (letrec ()
       (locals (a.1 b.2 c.3)
         (begin
           (set! a.1 0)
           (set! b.2 0)
           (if (if (= a.1 0) (= b.2 1) (false))
               (set! c.3 5)
               (begin (set! c.3 5) (set! c.3 (+ c.3 c.3))))
           c.3)))";
    test_helper(s, "c52.s", "10\n");
}

#[test]
fn compile53() {
    let s = "
    (letrec ([main$0 (lambda (x.1 y.2)
                       (locals (z.3)
                         (begin
                           (if (if (= x.1 1) (true) (= y.2 1))
                               (set! z.3 1)
                               (set! z.3 0))
                           (* z.3 5))))])
      (locals () (main$0 1 0)))";
    test_helper(s, "c53.s", "5\n");
}

#[test]
fn compile54() {
    let s = "
    (letrec ([main$0 (lambda (a.1 b.2)
                       (locals (c.3)
                         (begin
                           (set! c.3 
                             (if (if (= a.1 1) (true) (= b.2 1))
                                 1
                                 0))
                           (+ c.3 5))))])
      (locals () (main$0 0 1)))";
    test_helper(s, "c54.s", "6\n");
}

#[test]
fn compile55() {
    let s = "
    (letrec ([main$0 (lambda (a.1 b.2)
                       (locals ()
                         (begin
                           (if (if (= a.1 1) (= b.2 1) (true))
                               (set! a.1 1)
                               (set! b.2 0))
                           (set! b.2 (* b.2 10))
                           (set! a.1 (+ a.1 b.2))
                           a.1)))])
       (locals () (main$0 0 1)))";
    test_helper(s, "c55.s", "11\n");
}

#[test]
fn compile56() {
    let s = "
    (letrec ([main$0 (lambda (a.1 b.2)
                       (locals ()
                         (if (if (= a.1 1) (= b.2 1) (true)) 1 0)))])
      (locals () (main$0 1 0)))";
    test_helper(s, "c56.s", "0\n");
}

#[test]
fn compile57() {
    let s = "
    (letrec ([main$0 (lambda (a.1 b.2)
                       (locals ()
                         (if (if (= a.1 1) (= b.2 1) (true)) 1 0)))])
      (locals () (main$0 0 0)))";
    test_helper(s, "c57.s", "1\n");
}

#[test]
fn compile58() {
    let s = "
    (letrec ()
       (locals (a.1 b.2)
         (begin
           (set! a.1 1)
           (set! b.2 1)
           (if (if (= a.1 1) (= b.2 1) (true)) 1 0))))";
    test_helper(s, "c58.s", "1\n");
}

#[test]
fn compile59() {
    let s = "
    (letrec ()
      (locals (n.1 a.2 b.3 c.4)
        (begin
          (set! n.1 1)
          (begin
            (set! a.2 2)
            (begin
              (set! b.3 3)
              (set! n.1 (+ n.1 (if (= (+ n.1 b.3) b.3) 5 10)))
              (set! n.1 (+ n.1 b.3)))
            (set! n.1 (+ n.1 a.2)))
          (+ n.1 n.1))))";
    test_helper(s, "c59.s", "32\n");
}

#[test]
fn compile60() {
    let s = "
    (letrec ()
       (locals (a.1 b.2 c.3 d.4 e.5)
         (begin
           (set! a.1 1)
           (set! b.2 2)
           (set! c.3 3)
           (set! d.4 4)
           (set! e.5 5)
           (+ (+ (+ (+ e.5 d.4) c.3) b.2) a.1))))";
    test_helper(s, "c60.s", "15\n");
}
#[test]
fn compile61() {
    let s = "
    (letrec ()
       (locals (a.1 b.2 c.3 d.4 e.5 f.6)
         (begin
           (set! a.1 1)
           (set! b.2 2)
           (set! c.3 3)
           (set! d.4 4)
           (set! e.5 5)
           (set! f.6 6)
           (set! a.1 
             (if (> (+ a.1 d.4) f.6)
               (* a.1 (+ c.3 f.6))
               (* a.1 (+ b.2 e.5))
               ))
           a.1)))";
    test_helper(s, "c61.s", "7\n");
}

#[test]
fn compile62() {
    let s = "
    (letrec ([dot$0 (lambda (a.1 a.2 a.3 a.4 b.5 b.6 b.7 b.8)
                      (locals ()
                        (+ (* a.1 b.5) 
                           (+ (* a.2 b.6) 
                              (+ (* a.3 b.7) (* a.4 b.8))))))])
      (locals () (dot$0 2 4 6 8 1 3 5 7)))";
    test_helper(s, "c62.s", "100\n")
}

#[test]
fn compile63() {
    let s = "
    (letrec ([dot-double-first$1 (lambda (a.1 a.2 a.3 a.4 b.5 b.6 b.7 b.8)
                                   (locals ()
                                     (dot$0 (+ a.1 a.1) (+ a.2 a.2)
                                            (+ a.3 a.3) (+ a.4 a.4)
                                            b.5 b.6 b.7 b.8)))]
             [dot$0 (lambda (a.1 a.2 a.3 a.4 b.5 b.6 b.7 b.8)
                      (locals ()
                        (+ (* a.1 b.5) 
                           (+ (* a.2 b.6) 
                              (+ (* a.3 b.7) (* a.4 b.8))))))])
      (locals () (dot-double-first$1 2 4 6 8 1 3 5 7)))";
    test_helper(s, "c63.s", "200\n");
}

#[test]
fn compile64() {
    let s = "
    (letrec ([dot-double-first$1 (lambda (a.1 a.2 a.3 a.4 b.5 b.6 b.7 b.8)
                                   (locals ()
                                     (begin
                                       (set! a.1 (+ a.1 a.1))
                                       (set! a.2 (+ a.2 a.2))
                                       (set! a.3 (+ a.3 a.3))
                                       (set! a.4 (+ a.4 a.4))
                                       (dot$0 a.1 a.2 a.3 a.4
                                              b.5 b.6 b.7 b.8))))]
             [dot$0 (lambda (a.1 a.2 a.3 a.4 b.5 b.6 b.7 b.8)
                      (locals ()
                        (+ (* a.1 b.5) 
                           (+ (* a.2 b.6) 
                              (+ (* a.3 b.7) (* a.4 b.8))))))])
      (locals () (dot-double-first$1 2 4 6 8 1 3 5 7)))";
    test_helper(s, "c64.s", "200\n");
}

#[test]
fn compile65() {
    let s = "
    (letrec ()
      (locals (b.2 g.7 c.3 d.4 e.5 a.1 f.6 h.8 i.9 j.10 k.11)
        (begin
          (set! h.8 77)
          (set! i.9 88)
          (set! j.10 99)
          (set! k.11 111)
          (set! a.1 1)
          (set! b.2 2)
          (set! c.3 a.1)
          (set! d.4 4)
          (set! e.5 5)
          (set! f.6 b.2)
          (set! f.6 (+ f.6 c.3))
          (set! f.6 (+ f.6 d.4))
          (set! f.6 (+ f.6 e.5))
          (set! g.7 7)
          (set! f.6 (+ f.6 g.7))
          (set! f.6 (+ f.6 i.9))
          (set! f.6 (+ f.6 j.10))
          (set! f.6 (+ f.6 k.11))
          (+ f.6 h.8))))";
    test_helper(s, "c65.s", "394\n");
}

#[test]
fn compile66() {
    let s = "
    (letrec ([add1$3 (lambda () (locals () 1))]
             [high$4 (lambda (f.7)
                      (locals ()
                        (begin
                            (f.7)
                            (f.7))))])
      (locals ()
        (begin
          (high$4 add1$3))))";
    test_helper(s, "c66.s", "1\n");
}
