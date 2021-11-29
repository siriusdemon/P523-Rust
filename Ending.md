# P523-Rust | Ending

P523-Rust 基本上已经完成了，赶在 12 月之前 :) 

这说明，2021 年了，P523（2009 版）还是足够完整，可以继续学习的。

### P523

P523 原有 15 个 notes 和两个 challenges，网上只剩下的 14 个 notes，丢了 a12.pdf 和两个 challenges。根据网上的代码和注释，我补充了 A12 的内容，见 [guide/a12.md](./guide/a12.md)。

### P523 和 EoC

P523 采用自底向上的方式，它先一步一步做出一个 UIL，然后把 Scheme 编译到 UIL。EoC 采用自顶向下的方式，一开始就定义好 Scheme(Racket)，C 和 x86 整个编译链，然后不断地往里面加功能。

P523 的课程目标是实现一个 Scheme 子集，EoC 实现的是 Typed Racket。

P523 和 EoC 写的编译器都是 nanopass 的风格。

EoC 有配置的视频和教程，而且本身引用了很多资源。我觉得是个很不错的开始。详情见它的[官网](https://iucompilercourse.github.io/IU-P423-P523-E313-E513-Fall-2020/)。

### Rust-One-Piece

Rust-One-Piece 是一年前我所写的 build-your-own-x 风格的教程。遗憾的是写到一半就写不下去了，原因有很多，但我觉得其中一个重要的原因是，我一边写代码还在一边写博客，这样失败的概率就很高了。因为出错的成本变大了。当发现已经错得离谱的时候，已经太晚。

但 Rust-One-Piece 的前面几章还是可以看看的，如果一开始无法接受 P523 的话，可以试试。

这次完成了 P523，也算是圆了自己的一个小小心愿。

### Next One Piece

+ [EoC](https://iucompilercourse.github.io/IU-P423-P523-E313-E513-Fall-2020/)
+ P523
+ [nand2tetris](https://www.nand2tetris.org/)
+ [C311](https://cgi.luddy.indiana.edu/~c311/doku.php?id=assignments)
+ TAPL
+ [CIS 341](https://www.seas.upenn.edu/~cis341/current/)
+ [DCC 888](https://homepages.dcc.ufmg.br/~fernando/classes/dcc888/ementa/)
+ Computer Organization and Design (RISC-V Edition): The Hardware Software Interface
+ Computer Architecture: A Quantitative Approach (6th Edition)


> 一个海员说，他最喜欢的是起锚所激起的  那一片洁白的浪花。一个海员说，最使他高兴的是抛锚所发生的    那一阵铁链的喧哗…… 一个盼望出发；一个盼望到达。