#![feature(box_patterns)]

mod syntax;
mod parser;
mod compiler;
mod test;


use compiler::compile;


fn main() -> std::io::Result<()> {
    let s = "
    (letrec ((return$1 (lambda ()
                         (begin
                           (set! rax fv0)
                           (fv1))))
             (setbit3$0 (lambda ()
                          (begin
                            (set! fv0 (logor fv0 8))
                            (return$1)))))
      (begin
        (set! fv0 1)
        (set! fv1 r15)
        (setbit3$0)))";
    compile(s, "t.s")
}
