#![feature(box_patterns)]

mod syntax;
mod parser;
mod compiler;
mod test;


use compiler::compile;


fn main() -> std::io::Result<()> {
    let s = "(letrec () (let ([n.1 '#f]) (if (eq? n.1 n.1) '() (* '-1 '2))))";
    compile(s, "t.s")
}
