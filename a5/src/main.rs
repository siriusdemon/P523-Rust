#![feature(box_patterns)]

mod syntax;
mod parser;
mod compiler;
mod test;


use compiler::compile;


fn main() -> std::io::Result<()> {
    let s = "
    (letrec ()
    (locals (a.1 b.2 c.3)
    (begin
      (set! a.1 r8)
      (set! b.2 fv0)
      (set! c.3 (+ a.1 2))
      (if (< c.3 0) (nop) (set! c.3 (+ c.3 b.2)))
      (set! rax (+ c.3 1))
      (r15 rax rbp)))";
   compile(s, "t.s")
}
