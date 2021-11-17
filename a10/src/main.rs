#![feature(box_patterns)]

mod syntax;
mod parser;
mod compiler;
mod test;


use compiler::compile;


fn main() -> std::io::Result<()> {
    let s = "
    (letrec ()
      (let ([v (make-vector '3)])
        (begin 
          (vector-set! v '0 '10)
          (vector-set! v '1 '2)
          (vector-set! v '2 '4)
          (vector-ref v '1))))";
    compile(s, "t.s")
}
