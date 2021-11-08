#![feature(box_patterns)]

mod syntax;
mod parser;
mod compiler;
mod test;


use compiler::compile;


fn main() -> std::io::Result<()> {
    let s = "
    (letrec ([f$0 (lambda (h.1 v.2) (locals () (* h.1 v.2)))]
             [k$1 (lambda (x.1) (locals () (+ x.1 5)))]
             [g$2 (lambda (x.1) (locals () (+ 1 x.1)))])
      (locals (x.4 g.1)
        (begin
          (set! x.4 15)
          (k$1 (g$2 (begin (set! g.1 3) (f$0 g.1 x.4)))))))";
    compile(s, "t.s")
}
