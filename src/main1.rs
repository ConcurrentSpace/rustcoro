use std::arch::asm;

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

// 保存当前协程的上下文(此代码中未实现)
// 加载目标协程的栈指针(`rsp`)
// 通过`ret`指令跳转到目标协程上次暂停的位置
// 将指针`new_ctx`指向的内存地址加载到栈指针寄存器(rsp)
// 将`new_ctx`指针值放入通用寄存器作为汇编块的输入
// in - 这是一个输入操作数说明符，表示将一个 Rust 变量传入汇编代码
fn gt_switch(new_ctx: *const ThreadContext) {
    unsafe {
        asm!(
            "mov rsp, [{0} + 0x00]", // 偏移为 0
            "ret",
            // 将 Rust 变量 new_ctx 的值放入一个由编译器选择的寄存器中使其可以在接下来的汇编代码中使
            in(reg) new_ctx
        );
    }
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

        gt_switch(&mut ctx)
    };

    // println!("Hello, world!");
    // write(output, args) // todo: - this func when and why to add
}
