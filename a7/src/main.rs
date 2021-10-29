#![feature(box_patterns)]

mod syntax;
mod parser;
mod compiler;
mod test;


use compiler::compile;


fn main() -> std::io::Result<()> {
    let s = "
    (letrec ([f$0 (lambda () (locals () 80))])
      (locals (a.1 b.2)
        (begin
          (set! a.1 (f$0))
          (set! b.2 (f$0))
          (* a.1 b.2))))";
    compile(s, "t.s")
}
