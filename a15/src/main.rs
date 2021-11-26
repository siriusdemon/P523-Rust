#![feature(box_patterns)]
#![feature(hash_drain_filter)]

mod syntax;
mod parser;
mod compiler;
mod test;


use compiler::compile;


fn main() -> std::io::Result<()> {
    let s = "'#3(0)";
    compile(s, "t.s")
}


