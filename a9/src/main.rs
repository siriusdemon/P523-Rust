#![feature(box_patterns)]

mod syntax;
mod parser;
mod compiler;
mod test;


use compiler::compile;


fn main() -> std::io::Result<()> {
    let s = "
    (letrec ([f$0 (lambda (p.2) (- (mref p.2 8) (mref p.2 0)))])
      (let ([x.1 (alloc 16)])
        (begin
          (mset! x.1 0 73)
          (mset! x.1 8 35)
          (f$0 x.1))))";
    compile(s, "t.s")
}
