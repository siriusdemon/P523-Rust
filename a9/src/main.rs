#![feature(box_patterns)]

mod syntax;
mod parser;
mod compiler;
mod test;


use compiler::compile;


fn main() -> std::io::Result<()> {
    let s = "
    (letrec ()
      (let ([c.1 10] [a.2 5])
        (if (< a.2 c.1) a.2 c.1)))";
    compile(s, "t.s")
}
