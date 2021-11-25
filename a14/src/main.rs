#![feature(box_patterns)]
#![feature(hash_drain_filter)]

mod syntax;
mod parser;
mod compiler;
mod test;


use compiler::compile;


fn main() -> std::io::Result<()> {
    let s = "
    (letrec ([f.1 (lambda (x.2) 
                    (begin
                      (set! f.1 '10)
                      f.1))])
      (f.1))";
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
    compile(s, "t.s")
}


