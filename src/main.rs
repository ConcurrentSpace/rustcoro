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
    print!("hello wake up on a new stack");
    loop {}
}

// 保存当前协程的上下文(此代码中未显示)
// 加载目标协程的栈指针(`rsp`)
// 通过`ret`指令跳转到目标协程上次暂停的位置
// 将指针`new_ctx`指向的内存地址加载到栈指针寄存器(rsp)
// 将`new_ctx`指针值放入通用寄存器作为汇编块的输入
unsafe fn gt_switch(new_ctx: *const ThreadContext) {
    asm!(
        "mov rsp, [{0}]",
        "ret",
        in(reg) new_ctx,
        options(nostack)
    );
}

// - `&mut T`可以隐式转换为`*const T`(不可变原始指针)
// - Rust允许可变引用到不可变指针的自动转换
// - 这种转换是安全的，因为不会通过不可变指针修改数据
fn main() {
    let mut ctx = ThreadContext::default();
    let mut stack = vec![0_u8; STACK_SIZE as usize];

    unsafe {
        let stack_bottom = stack.as_mut_ptr().offset(STACK_SIZE);
        let sb_aligned = (stack_bottom as usize & !15) as *mut u8;
        std::ptr::write(sb_aligned.offset(-16) as *mut u64, hello as u64);
        ctx.rsp = sb_aligned.offset(-16) as u64;
        gt_switch(&mut ctx)
    };

    println!("Hello, world!");
    // write(output, args) // todo: - this func when and why to add
}
