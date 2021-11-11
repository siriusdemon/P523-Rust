## P523-Rust


P523 is a classic compiler course taught by R. Kent Dybvig. This repo implements the course using Rust, provides a framework to help you master P523.

You are expected to start from [adventure](./adventure), write your own code until you finally arrive at A15. It is not really scare as it sounds, since if you are able to pass the A1, you are able to pass A2, A3 and so on.

The P523 PDF files spreaded around the Internet is the main materials you have to read again and again. You can find them [here](https://github.com/Booob/P523). Asides, the reference codes serve as hints. 


### Adventure

Adventure is exactly same as a1. You are expected to master every line of code here.

```rs
main.rs             : 10
syntax.rs           : 56
parser.rs           : 181
compiler.rs         : 113
test.rs             : 132
----------------------------
total               : 492
```


Firstly, you should read the main.rs. It includes other modules, use a single interface `compile` to compile the program. The result is saved in "t.s".

Then, read the syntax.rs. There are two enums there. One is Expr, another is Asm. Your goal is to transform the Expr code into Asm code. The trait `Display` impl for Expr is for debug purpose, and for Asm is for generating assemble code.

And then you are ready to read parser.rs. Or you can just skip it if you don't care. You can copy mine into your folder. The parser simply transforms the string-form program into an abstract-syntax-tree Expr.

There are two classes defined in parser.rs, Scanner and Parser. Scanner travers through the string-form program and tokenizes it. A token consists of four parts: the string, the global index, line and column. The Scanner skips comment and newline. Parser parse the token stream using recursive-descent.

It is time to write your own Pass in compiler.rs. Since A1 is really simple, you are expected to understand the whole transformation. After that, you are really really ready for the P523 adventure.

Tests, you can understand them absolutely.

If you have any questions, please open a new issue at this repo.

By the way, [yscheme.ss](./yscheme.ss) is a good reference.

Have a Good Time!
