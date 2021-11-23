#![feature(box_patterns)]

mod syntax;
mod parser;
mod compiler;
mod test;


use compiler::compile;


fn main() -> std::io::Result<()> {
    let s = "((lambda (x.1) (+ x.1 '1)) '41)";
    let s = "
    ((lambda (y.2)
        ((lambda (f.1) (f.1 (f.1 y.2)))
            (lambda (x.3) (+ x.3 '1))))
     '3)";
    compile(s, "t.s")
}


