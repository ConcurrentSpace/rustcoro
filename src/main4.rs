use std::arch::global_asm;
use std::io::Write;

mod fiber;

// todo: fix home codelldb not install -> debug problem

// .option prefix 用于控制符号名称的前缀处理, 在某些平台上（特别是 RISC-V），符号名可能需要特定前缀
// .option norelax 禁用汇编器的重定位优化, 特别在 RISC-V 架构中很重要
global_asm!(
    include_str!("switch.S"),
    options(att_syntax) // 这里你可以修改为 intel | at&t 语法
);

const STACK_SIZE: isize = 1024;
static mut S_PTR: *const u8 = 0 as *const u8;

fn print_stack(filename: &str) {
    let mut file = std::fs::File::create(filename).unwrap();
    unsafe {
        for i in (0..STACK_SIZE).rev() {
            // println!("index = {}", i);
            writeln!(
                file,
                "{i}: mem: {}, value: {}",
                S_PTR.offset(i as isize) as usize,
                *S_PTR.offset(i as isize)
            )
            .expect("error writing to file.");
        }
    }
}

// #[repr(C)]
// 指定结构体使用C语言的内存布局
// 这是必要的，因为该结构体可能会在Rust和汇编代码之间传递
// 保证字段顺序和内存对齐符合预期
// todo: - 查找调用约定
#[derive(Debug, Default)]
// 不会直接造成 unsafe，也就是说使用结构体不会使用 unsafe，直接规定一种内存布局形式，但是用指针地址给寄存器赋值的时候必然会造成 unsafe
// todo: - 打印内存布局，显示差异
#[repr(C)]
struct ThreadContext {
    rsp: u64,
    r15: u64,
    r14: u64,
    r13: u64,
    r12: u64,
    rbx: u64,
    rbp: u64,
}

fn hello() -> ! {
    // println!("hello wake up on a new stack");
    print_stack("after.txt"); // 切换到 hello() 后的栈状态

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
    let stack_ptr = stack.as_mut_ptr();

    // 为什么不用像之前那样 16 字节对齐
    // 虽然STACK_SIZE - 16可能不是16的倍数
    // 但现代CPU对mov指令通常有较好的非对齐访问支持
    // 作为学习示例可以工作，但生产代码应该保持对齐
    unsafe {
        S_PTR = stack_ptr;
        std::ptr::write(stack_ptr.offset(STACK_SIZE - 16) as *mut u64, hello as u64);
        print_stack("before.txt"); // 打印 main() 函数设置的初始栈状态
        ctx.rsp = stack_ptr.offset(STACK_SIZE - 16) as u64;
        println!("rsp = {}", ctx.rsp);
        gt_switch(&mut ctx) // todo: - jump where
    };

    fiber::run();
}

// pub fn main() {
//     let mut ctx = ThreadContext::default();
//     let mut stack = vec![0_u8; STACK_SIZE as usize];
//     let stack_ptr = stack.as_mut_ptr();
//     unsafe {
//         let stack_bottom = stack.as_mut_ptr().offset(STACK_SIZE);
//         let sb_aligned = (stack_bottom as usize & !15) as *mut u8;
//         S_PTR = sb_aligned;
//         std::ptr::write(stack_ptr.offset(STACK_SIZE - 16) as *mut u64, hello as u64);
//         print_stack("before.txt");
//         ctx.rsp = stack_ptr.offset(STACK_SIZE - 16) as u64;
//         gt_switch(&mut ctx);
//     }
// }
