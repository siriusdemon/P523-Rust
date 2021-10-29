#![feature(box_patterns)]

mod syntax;
mod parser;
mod compiler;
mod test;


use compiler::compile;


fn main() -> std::io::Result<()> {
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
    compile(s, "t.s")
}
