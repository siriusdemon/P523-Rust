#![feature(box_patterns)]

mod syntax;
mod parser;
mod compiler;
mod test;


use compiler::compile;


fn main() -> std::io::Result<()> {
    let s = "
    (letrec ([add1$3 (lambda () (locals () 1))]
             [high$4 (lambda (f.7)
                      (locals ()
                        (begin
                            (f.7)
                            (f.7))))])
      (locals ()
        (begin
          (high$4 add1$3))))";
    compile(s, "t.s")
}
