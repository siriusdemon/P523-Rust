# P523-Rust
P523 经典编译器课程的 Rust 引导

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

### week3

随着开发的推进，在 parser 和 scanner 有一些小的改进，散落在各个角落，但仍不影响原来的架构。

修改 syntax 以后，从 letrec 开始改起。新增 parse_locate

需要把所有的 special form 都写上，不然会有奇怪的 bug

如果遇到了 '('，有两种可能，一种是忘记 remove_top 将右边的括号删除，另一种是因为 special form 没覆盖导致的。

在 P523，parse 是平淡无奇的，但在这里，parse 是惊心动魄的。:D


+ expose-basic-blocks

这是我遇到的第一个有难度的 PASS。虽然分析出解法不容易，但代码的实现却异常的简单。

写代码前应该先想清楚。
观察输入的语法，发现

+ tail, effect, pred 当中都可以出现 if 和 begin，这两种语句是重点要处理的
+ 新的语法中，if 只能出现在 tail，故知，effect 和 pred 中的 if 要转换成 tail
+ tail 是处理入口，因此 tail_helper 先写，它应该返回一个 tail
+ pred_helper 需要生成 blocks 和跳转，因此，它的返回也是一个 tail，跳往新的 blocks
+ effects 只能出现在 begin 中，故知 effects_helper 需要返回一个 Begin。
+ effect_helper 注意，effect 并不能作为一个 block 的结尾，所以它需要传入一个 tail，它的返回也应该是一个 tail。

给自己预留点思考的时间吧！学而不思则罔。

+ optimize-jump

引入新的语义 If1 来表示单支 if 语句。为什么不直接编译到 Asm 中的 Jmpif 呢？理由是，我们希望减轻 CompileToAsm 的负担，这样，当我们想换一个后端，比如 Riscv 的时候，相对就好写一些。

从 P523 的 flatten-program 中剥离。

situation1:

(if (< rax 3) (l$1) (l$2))
(if (< rax 3) (l$1))
l$2 ...

对汇编代码进行拓扑排序，减少不必要的 jmp，理应能够提升性能。但现在先不考虑这么多的优化吧！


+ CompileToAsm
cmpq S2, S1 => S1 - S2


locate 可以不存在，A2 的 PASS 能够通过，较 P523 更宽松。

NOTE:

这一节，if-test 会在输出到文件的时候变成　if_test，因为 Gna Assembler 不允许 if-test 这样的标签。

这一节的　Jump 还有可优化的地方。比如一些没有必要的空 block。

且 cmpq 没有对常数进行检查。但更好的做法应该是进行 constant-folding


### week4

parse 过于简单，只是把 locate 转换成 locals，非常适合作为练习。到后面，也许 parse 会越来越简单，因为更多的语法变成了临时语法。

可以多设计一个 Funcall 带参数的，这样不影响原来的 Funcall，但是，我决定修改原来的 Funcall，使之带一个参数列表，并修改受影响的所有的代码。


register-assign


1. 取一个冲突最少的一个变量
2. 将之从冲突图中移除
3. 递归
4. 为刚刚所选的变量分配寄存器


discard-call-live

call-live 本身记录的调用的顺序，传递了那些需要用到的参数。看似无用，实为关键。
