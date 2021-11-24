#![feature(box_patterns)]

mod syntax;
mod parser;
mod compiler;
mod test;


use compiler::compile;


fn main() -> std::io::Result<()> {
    let s = "
    (let ([f.1 (lambda () '(1 . 2))])
        (eq? (f.1) (f.1)))";
    compile(s, "t.s")
}


