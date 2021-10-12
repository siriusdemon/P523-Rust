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

ExposeFrameVar 被融合进 CompileToAsm

Scanner 对每个 Token 增加了行与列的属性，并且跳过注释

Expr 新增 letrec, label, lambda, disp

Asm 新增 Deref

fv is not allowed in funcall

leaq

### 3

修改 syntax 以后，从 letrec 开始改起。新增 parse_locate

需要把所有的 special form 都写上，不然会有奇怪的 bug

如果遇到了 '('，有两种可能，一种是忘记 remove_top 将右边的括号删除，另一种是因为 special form 没覆盖导致的。