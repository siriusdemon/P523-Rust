## P523-Rust


P523 is a classic compiler course taught by R. Kent Dybvig. This repo implements the course using Rust, provides a framework to help you master P523.

There will be two branch of this repo, one will be designed as a homework. You are asked to write some code to pass all the tests. Another one is same as the previous one but with full code, useful when you stuck.

The P523 PDF files spreaded around the Internet is the main materials you have to read again and again. Asides, this repo provides some useful hints. The code framework serves as a hint too.

I am going to explain the framework a little bit when I finish.

Note: As you proceeds, the framework also evolves. Some patterns appear and certain code are modified to abstract them. But the whole framework is still  understandable. So this course will be an adventure! Enjoy it!

Note: You may find some code just work and not 100% correct. If it happens, refer to the rest assignment, it may be fixed.

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

Note that, the expose-frame-var is merged naturally into CompileToAsm. Of course, you are free to make your own one.

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

The register allocator in this week is very naive or just wrong if you like.

### Week5

+ UncoverFrameConflict
+ IntroduceAllocationForm
+ SelectInstruction
+ AssignFrame
+ FinalizeFrameLocations

select-instruction is really challenge. Convince yourself that the following cases cover the (set a (op b c))
```lisp
(set a (op b c))
(set a (op a b))
(set a (op b a))
(set a (op imm a))
(set a (op a imm))
(set a (op imm b))
(set a (op b imm))
```

move-relation optimization is skipped

### week6

+ RemoveComplexOpera
+ FlattenSet
+ ImposeCallingConvention

since we have no expose-frame-variable at all, we need to modify the CompileToAsm, fv_to_deref

(value value*) is not supported now. Because I think it is a scheme-feature, not a UIL feature.


### Hint

Pen and paper are your friends.