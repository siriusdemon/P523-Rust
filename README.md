# P523-Rust
P523 Course in Rust

### 工具

+ Rust Nightly
+ GCC


### 流程

1. 定义 syntax
2. 修改 parser
3. 增减 test
4. 修改 compiler
5. 增减 test

### week1

+ ParsePass
将字符串转换成语法树

+ CompileToAsmPass
将 Scheme 的表达式转换成 x86-64 的汇编格式

+ GenerateAsmPass
将汇编代码写到文件中

### week2

Scanner 对每个 Token 增加了行与列的属性

Expr 新增 letrec, label, lambda, disp

Asm 新增 Deref

fv is not allowed in funcall