#![feature(box_patterns)]

mod syntax;
mod parser;
mod compiler;
mod test;


use compiler::compile;


fn main() -> std::io::Result<()> {
    let s = "(begin (set! rax 8) (set! rcx 3) (set! rax (- rax rcx)))"; 
    compile(s, "t.s")
}
