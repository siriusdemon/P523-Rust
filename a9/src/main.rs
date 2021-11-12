#![feature(box_patterns)]

mod syntax;
mod parser;
mod compiler;
mod test;


use compiler::compile;


fn main() -> std::io::Result<()> {
    let s = "
    (letrec ([sum$1 (lambda (x.1 y.2 z.3 w.4)
                      (+ x.1 (+ y.2 (+ z.3 w.4))))])
      (let ([a.6 (alloc 8)])
        (sum$1 (begin (mset! a.6 0 1) (mref a.6 0))
               (begin (mset! a.6 0 2) (mref a.6 0))
               (begin (mset! a.6 0 3) (mref a.6 0))
               (begin (mset! a.6 0 4) (mref a.6 0)))))";
    compile(s, "t.s")
}
