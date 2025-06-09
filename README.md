## 为什么我用这种方式导入汇编，必须要用 intel 语法？

使用 global_asm! 宏导入汇编时，Rust 默认使用 Intel 语法的原因如下：

LLVM 后端的默认选择

Rust 使用 LLVM 作为其后端
LLVM 的内联汇编默认使用 Intel 语法
这是为了保持与 LLVM 的一致性
语法一致性

Rust 的 asm! 宏也使用 Intel 语法
保持 global_asm! 和 asm! 的语法一致性，降低学习成本
不过，你可以显式指定使用 AT&T 语法，方法是：

```asm
.att_syntax
.global gt_switch
.section .text

gt_switch:
    mov (%rdi), %rsp
    ret
```

text
对比两种语法的区别：

特性	Intel 语法	AT&T 语法
操作数顺序	mov dest, src	mov src, dest
寄存器命名	rax	%rax
立即数	42	$42
内存引用	[rax]	(%rax)
选择建议：

如果使用 global_asm!：优先使用 Intel 语法
如果使用 build.rs：两种语法都可以，但要保持一致

## refs

- https://zhuanlan.zhihu.com/p/101061389

