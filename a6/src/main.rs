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
    let s = "
    (letrec ()
      (locals (a.1 b.2)
        (if (+ (+ a.1 (+ b.2 1)) (+ 10 b.2)) (if (= a (+ b c)) 4 6) 19)))";
    
    compile(s, "t.s")
}
