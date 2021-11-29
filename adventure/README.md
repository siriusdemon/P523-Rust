# Adventure

In order to make your adventure more fun. I remove some code and leave some test fail. You are expected to fix them. Don't worry, they are very trivial.

### Intro

P523 的前半段是在实现一个类似 C 语言的 UIL（通用中间语言），后半段是在实现 Scheme。P523-Rust 的策略是，定义 Scheme、Expr、Asm 三种语言。前半段，实现从 Expr 编译到 Asm，后半段，实现从 Scheme 编译到 Expr。 

```rs
main.rs             : 10
syntax.rs           : 56
parser.rs           : 181
compiler.rs         : 113
test.rs             : 132
----------------------------
total               : 492
```


首先，看看 main.rs，它引入了其他模块，并给出了编译的示例。结果保存在 "t.s"。

然后，读下 syntax.rs，目前里面有两个 enums。一个是 Expr，一个是 Asm。你的目标是将 Expr 的代码编译到 Asm。Expr 对 `Display` 的实现纯属为了方便调试，而 Asm 的实现则是为了在最后生成汇编代码。

接着，读下 parser.rs。如果你不想花时间在 parser 上，你可以跳过它，并在每次作业开始前复制我写好的 syntax.rs 和 parser.rs。这个 parser.rs 的功能仅仅是将字符串形式的代码转换成抽象语法树。

parser.rs 中有两个主要的结构, Scanner and Parser. Scanner 会遍历字符串并进行词法分析。 每个 token 包括四个部分: 内容本身, 全局索引, 行与列位置。Scanner 会跳过注释和换行。Parser 使用的是递归下降的方法进行解析，解析的入口是去判断当前的 S 表达式是一个原子还是一个列表，然后对应做处理。

读完 parser.rs 之后，你就对整个框架有清楚的了解了，现在可以写你自己的 pass。因为 A1 很简单，你应该要完全理解代码的转换过程。在这之后，你就真的可以开始你自己的冒险了。你完全不必拘泥于我的写法。

Adventure 提供了一些测试，你应该看一下就能理解，并且也能理解到框架的原理。所以推荐你去看一下。

如果你有任何问题，可以在 Github 上进行提问。