#![feature(box_patterns)]

mod syntax;
mod parser;
mod compiler;
mod test;


use compiler::compile;


fn main() -> std::io::Result<()> {
    let s = "
    (letrec ([main$0 (lambda (a.1 b.2)
                       (locals (c.3)
                         (begin
                           (set! c.3 
                             (if (if (= a.1 1) (true) (= b.2 1))
                                 1
                                 0))
                           (+ c.3 5))))])
      (locals () (main$0 0 1)))";
    let s = "
    (letrec ()
      (locals () 
        (begin
          (r15 (+ (* x.2 x.5) 7) (sra x.1 3)))))";
   compile(s, "t.s")
}
