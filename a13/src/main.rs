#![feature(box_patterns)]

mod syntax;
mod parser;
mod compiler;
mod test;


use compiler::compile;


fn main() -> std::io::Result<()> {
    let s = "((lambda (x.1) (+ x.1 '1)) '41)";
    let s = "
    (let ([f.3 (lambda (x.1)
                  (lambda (y.2)
                    (+ x.1 y.2)))])
        ((f.3 '1) '2))";
    compile(s, "t.s")
}


