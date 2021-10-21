#![feature(box_patterns)]

mod syntax;
mod parser;
mod compiler;
mod test;


use compiler::compile;


fn main() -> std::io::Result<()> {
    let s = "
    (letrec ([f$1 (lambda ()
                     (locals (x.1 y.2 z.3)
                       (begin
                         (set! x.1 1)
                         (set! y.2 2)
                         (set! rax (+ x.1 y.2))
                         (r15 rax rcx rdx rbx rbp rdi rsi r8 r9 r10
                              r11 r12 r13 r14))))])
       (locals () (f$1 rbp r15)))";
    let s = "
    (letrec ()
    (locals (a.1 b.2 c.3)
    (begin
      (set! a.1 r8)
      (set! b.2 fv0)
      (set! c.3 (+ a.1 2))
      (if (< c.3 0) (nop) (set! c.3 (+ c.3 b.2)))
      (set! rax (+ c.3 1))
      (r15 rax rbp)))";
   compile(s, "t.s")
}
