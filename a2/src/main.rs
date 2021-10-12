#![feature(box_patterns)]

mod syntax;
mod parser;
mod compiler;
mod test;


use compiler::compile;


fn main() -> std::io::Result<()> {
    let s = "
    (letrec ((div$0 (lambda ()
                      (begin
                        (set! fv2 (sra fv2 1))
                        (div$1))))
             (div$1 (lambda ()
                      (begin
                        (set! rax fv2)
                        (fv0)))))
      (begin
        (set! fv0 r15)
        (set! rax div$0)
        (set! fv1 rax)
        (set! fv2 64)
        (fv1)))";
    compile(s, "t.s")
}
