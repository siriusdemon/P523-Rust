#![feature(box_patterns)]

mod syntax;
mod parser;
mod compiler;
mod test;


use compiler::compile;


fn main() -> std::io::Result<()> {
    let s = "
    (letrec ((f$1 (lambda ()
                    (locate ((x.1 r8) (y.2 r9))
                        (if (if (= x.1 1) (true) (> y.2 1000))
                            (begin (set! rax y.2) (r15))
                            (begin
                                (set! y.2 (* y.2 2))
                                (set! rax x.1)
                                (set! rax (logand rax 1))
                                (if (= rax 0) (set! y.2 (+ y.2 1)) (nop))
                                (set! x.1 (sra x.1 1))
                                (f$1)))))))
        (begin (set! r8 3) (set! r9 10) (f$1)))";
    compile(s, "t.s")
}
