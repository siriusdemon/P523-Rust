#![feature(box_patterns)]

mod syntax;
mod parser;
mod compiler;
mod test;


use compiler::compile;


fn main() -> std::io::Result<()> {
    let s = "(letrec ((f$1 (lambda () (begin 
                                        (set! fv0 rax)
                                        (set! rax (+ rax rax))
                                        (set! rax (+ rax fv0))
                                        (r15)))))
                (begin 
                  (begin
                    (set! rax 17)
                    (f$1))))";

    let s = "(letrec () (begin (set! rax 8) (set! rcx 3) (set! rax (- rax rcx)) (r15)))"; 
    compile(s, "t.s")
}
