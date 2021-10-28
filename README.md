## P523-Rust


P523 is a classic compiler course taught by R. Kent Dybvig. This repo implements the course using Rust, provides full documentation and materials to help you master P523.

There will be two branch of this repo, one will be designed as a homework framework. You are asked to write some code to pass all the tests. Another one is same as the previous one but with full code, useful when you stuck.

The P523 PDF files spreaded around the Internet is the main materials you have to read again and again. Asides, this repo provides some useful hints and the homework framework serves as a hint too.

I am going to explain the homework framework a little bit when I finish.


Note: As you proceeds, the framework also evolves. Some patterns appear and certain code are modified to abstract them. But the whole framework is still  understandable. So this course will be an adventure! Enjoy it!

----------------------------------------------------

### Week1

+ ParseExpr
+ CompileToAsm
+ GenerateAsm


### Week2

+ ParseExpr
+ FlattenProgram
+ CompileToAsm
+ GenerateAsm

### Week3

+ ParseExpr
+ FinalizeLocations
+ ExposeBasicBlocks
+ OptimizeJump
+ CompileToAsm

tests in this week record some case which will be solved in Week5.

### Week4

+ ParseExpr
+ UncoverRegisterConflict
+ AssginRegister
+ DiscardCallLive


### Week5

+ UncoverFrameConflict
+ IntroduceAllocationForm
+ SelectInstruction

convince yourself that the following cases cover the (set a (op b c))
```lisp
(set a (op b c))
(set a (op a b))
(set a (op b a))
(set a (op imm a))
(set a (op a imm))
(set a (op imm b))
(set a (op b imm))
```

move-relation optimization is skiped

### week6

since we have no expose-frame-variable at all, we need to modify the CompileToAsm, fv_to_deref


### 

士不可以不弘毅，任重而道远。仁以为己任，不亦重乎？死而后已，不亦远乎？

如果海洋注定要决堤，就让所有的苦水都注入我心中；如果陆地注定要上升，就让人类重新选择生存的峰顶。

愿我走过的苦难你不必经历，愿我已有的幸福你触手可及。

理想开花，桃李要结甜果；理想抽芽，榆杨会有浓荫。请乘理想之马，挥鞭从此启程，路上春色正好，天上太阳正晴。

艰难困苦，玉汝于成。


### Hint

Pen and paper are your friends.