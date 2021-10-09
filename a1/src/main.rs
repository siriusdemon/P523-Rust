#![feature(box_patterns)]

mod syntax;
mod parser;
mod compiler;
mod test;


use compiler::compile;


fn main() {
    let s = "(begin (set! rax 8) (set! rcx 3) (set! rax (- rax rcx)))"; 
    compile(s);
}
