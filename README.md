## P523-Rust


P523 is a classic compiler course taught by R. Kent Dybvig. This repo implements the course using Rust, provides full documentation and material to help you master P523.

There will be two branch of this repo, one will be designed as a homework framework. You are asked to write some code to pass all the tests. Another one is same as the previous with full code, useful when you stuck.

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

### Week4

+ ParseExpr
+ UncoverRegisterConflict
+ AssginRegister
+ DiscardCallLive
