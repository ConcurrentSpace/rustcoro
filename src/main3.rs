// https://zhuanlan.zhihu.com/p/101061389

use std::arch::global_asm;

// .option prefix 用于控制符号名称的前缀处理, 在某些平台上（特别是 RISC-V），符号名可能需要特定前缀
// .option norelax 禁用汇编器的重定位优化, 特别在 RISC-V 架构中很重要
global_asm!(
    include_str!("switch.S"),
    options(att_syntax) // 这里你可以修改为 intel | at&t 语法
);

const STACK_SIZE: isize = 48;

// #[repr(C)]
// 指定结构体使用C语言的内存布局
// 这是必要的，因为该结构体可能会在Rust和汇编代码之间传递
// 保证字段顺序和内存对齐符合预期
#[derive(Debug, Default)]
#[repr(C)]
struct ThreadContext {
    rsp: u64,
}

fn hello() -> ! {
    println!("hello wake up on a new stack");
    loop {}
}

unsafe extern "C" {
    fn gt_switch(new_ctx: *const ThreadContext);
}

// `&mut T`可以隐式转换为`*const T`(不可变原始指针)
// Rust允许可变引用到不可变指针的自动转换
// 这种转换是安全的，因为不会通过不可变指针修改数据
fn main() {
    let mut ctx = ThreadContext::default();
    let mut stack = vec![0_u8; STACK_SIZE as usize];

    unsafe {
        // 获取栈底指针,栈是向下增长的，所以"栈底"实际上位于最高的内存地址，栈底指针位于最高地址处
        let stack_bottom_ptr = stack.as_mut_ptr().offset(STACK_SIZE); 

        // 这行代码确保地址是16字节对齐的（x86_64 ABI的要求）：
        //     - !15 创建一个除最后4位外都为1的位掩码
        //     & 操作清除最后4位，实际上是将地址向下舍入到最近的16字节边界
        let sb_aligned = (stack_bottom_ptr as usize & !15) as *mut u8;

        // 第三行将函数指针写入栈
        std::ptr::write(sb_aligned.offset(-16) as *mut u64, hello as u64);

        // 将 rsp 寄存器指向这个新位置，这里将存储函数的返回地址 - 栈顶指针
        ctx.rsp = sb_aligned.offset(-16) as u64; 

        for i in 0..STACK_SIZE {
            println!("mem: {}, val: {}", sb_aligned.offset(-i as isize) as usize, *sb_aligned.offset(-i as isize));
        }

        gt_switch(&mut ctx)
    };

    // println!("Hello, world!");
    // write(output, args) // todo: - this func when and why to add
}
