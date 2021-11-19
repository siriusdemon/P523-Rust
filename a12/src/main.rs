#![feature(box_patterns)]

mod syntax;
mod parser;
mod compiler;
mod test;


use compiler::compile;


fn main() -> std::io::Result<()> {
    let s = "
    (let ([x.1 '3])
      (letrec ([f.2 (lambda () x.1)])
        (f.2)))";
    compile(s, "t.s")
}


