#![feature(box_patterns)]

mod syntax;
mod parser;
mod compiler;
mod test;


use compiler::compile;


fn main() -> std::io::Result<()> {
    let s = "((lambda (x.1) (+ x.1 '1)) '41)";
    let s = "
    (letrec ([map.1 (lambda (f.2 ls.3)
                    (if (null? ls.3)
                        '()
                        (cons (f.2 (car ls.3))
                            (map.1 f.2 (cdr ls.3)))))])
        (let ([mulx.4 (lambda (x.5)
                        (lambda (y.6)
                            (* x.5 y.6)))])
            (map.1 (mulx.4 '7)
                (map.1 (lambda (z.8) (+ z.8 '1))
                    (cons '1 (cons '2 '()))))))";
    compile(s, "t.s")
}


