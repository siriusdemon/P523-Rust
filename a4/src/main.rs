#![feature(box_patterns)]

mod syntax;
mod parser;
mod compiler;
mod test;


use compiler::compile;


fn main() -> std::io::Result<()> {
    let s = "
     (letrec ()
       (locals (a.1 b.2)
         (begin
           (set! a.1 1)
           (set! b.2 0)
           (if (if (= a.1 1) (= b.2 1) (true))
               (set! rax 1)
               (set! rax 0))
           (r15 rax))))";
    compile(s, "t.s")
}
