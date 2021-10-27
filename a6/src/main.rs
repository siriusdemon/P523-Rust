#![feature(box_patterns)]

mod syntax;
mod parser;
mod compiler;
mod test;


use compiler::compile;


fn main() -> std::io::Result<()> {
    let s = "
    (letrec ()
      (locals (a.1)
        (begin
          (set! a.1 10)
          (if (< 7 a.1)
              (nop)
              (set! a.1 (+ a.1 a.1)))
          a.1)))";
   compile(s, "t.s")
}
