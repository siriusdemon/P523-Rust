#![feature(box_patterns)]

mod syntax;
mod parser;
mod compiler;
mod test;


use compiler::compile;


fn main() -> std::io::Result<()> {
    let s = "
    (let ([x.1 '3])
      (letrec ([f.2 (lambda (y.3) (+ y.3 x.1))])
        (f.2 '10)))";
    let s = "    
    (let ([a.1 (letrec ([f$0 (lambda () '80)]) (f$0))]
          [b.2 (letrec ([g$1 (lambda () '50)]) (g$1))])
      (* a.1 b.2))";
    compile(s, "t.s")
}


