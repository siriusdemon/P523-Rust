#![feature(box_patterns)]

mod syntax;
mod parser;
mod compiler;
mod test;


use compiler::compile;


fn main() -> std::io::Result<()> {
    let s = "
    (letrec ([sum$1 (lambda (x.1 y.2 z.3 w.4)
                      (locals ()
                        (+ x.1 (+ y.2 (+ z.3 w.4)))))])
      (locals (a.1)
        (sum$1 (begin (set! a.1 1) a.1)
               (begin (set! a.1 2) a.1)
               (begin (set! a.1 3) a.1)
               (begin (set! a.1 4) a.1))))";
    compile(s, "t.s")
}
