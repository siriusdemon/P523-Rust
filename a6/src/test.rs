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
    let filename = "c1.s";
    compile(s, filename);
    let r = run_helper(filename);
    assert_eq!(r.as_str(), "6\n");
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
    let filename = "c2.s";
    compile(s, filename);
    let r = run_helper(filename);
    assert_eq!(r.as_str(), "10\n");
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
    let filename = "c3.s";
    compile(s, filename);
    let r = run_helper(filename);
    assert_eq!(r.as_str(), "32\n");
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
    let filename = "c4.s";
    compile(s, filename);
    let r = run_helper(filename);
    assert_eq!(r.as_str(), "310\n");
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
    let filename = "c5.s";
    compile(s, filename);
    let r = run_helper(filename);
    assert_eq!(r.as_str(), "32\n");
}

// I leave this test here to remain myself that the register allocator is not optimized.
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
    let filename = "c6.s";
    compile(s, filename);
    let r = run_helper(filename);
    assert_eq!(r.as_str(), "19\n");
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
    let filename = "c7.s";
    compile(s, filename);
    let r = run_helper(filename);
    assert_eq!(r.as_str(), "10\n");
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
    let filename = "c8.s";
    compile(s, filename);
    let r = run_helper(filename);
    assert_eq!(r.as_str(), "820\n");
}