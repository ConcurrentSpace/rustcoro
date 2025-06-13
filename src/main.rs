use std::arch::global_asm;

// options(att_syntax) // 这里你可以修改为 raw | att_syntax 语法
// options(raw)
global_asm!(include_str!("switch.s"), options(att_syntax));

const DEFAULT_STACK_SIZE: usize = 1024 * 1024 * 2;
const MAX_THREADS: usize = 4;
static mut RUNTIME: usize = 0;

// 每个寄存器使用固定 8 字节偏移
#[derive(Debug, Default)]
#[repr(C)]
struct ThreadContext {
    rsp: u64, // 0x00 Stack Pointer 栈指针寄存器，指向当前栈顶位置，每个协程有自己的栈空间，切换时必须保存/恢复，确保协程恢复后能继续使用自己的栈
    r15: u64, // 0x08
    r14: u64, // 0x10
    r13: u64, // 0x18
    r12: u64, // 0x20
    rbx: u64, // 0x28 通用寄存器，常用于存储基地址或计算
    rbp: u64, // 0x30 Base Pointer 基指针寄存器，用于访问栈帧中的局部变量和参数，维护函数调用栈的结构，在调试和栈回溯中特别重要
    thread_ptr: u64,
}

#[derive(PartialEq, Eq, Debug)]
enum State {
    Available, // 表示线程可用，并且可以根据需要分配任务
    Running,   // 意味着线程正在运行
    Ready,     // 意味着线程已准备好继续前进和恢复执行，已经调度过了等待恢复
}

struct Thread {
    id: usize,
    stack: Vec<u8>,
    ctx: ThreadContext,
    state: State,
    task: Option<Box<dyn FnOnce()>>,
}

impl Thread {
    fn new(id: usize) -> Self {
        Thread {
            id,
            stack: vec![0_u8; DEFAULT_STACK_SIZE],
            ctx: ThreadContext::default(),
            state: State::Available,
            task: None,
        }
    }
}

pub struct Runtime {
    threads: Vec<Thread>,
    current: usize,
}

impl Runtime {
    fn new() -> Self {
        let base_thread = Thread {
            id: 0,
            stack: vec![0_u8; DEFAULT_STACK_SIZE],
            ctx: ThreadContext::default(),
            state: State::Running,
            task: None,
        };

        let mut threads = vec![base_thread];
        threads[0].ctx.thread_ptr = &threads[0] as *const Thread as u64;
        let mut avaliable_threads: Vec<Thread> = (1..MAX_THREADS).map(|i| Thread::new(i)).collect();
        threads.append(&mut avaliable_threads);

        // println!("total threads len = {}", threads.len());

        Runtime {
            threads: threads,
            current: 0,
        }
    }

    fn init(&self) {
        unsafe {
            let r_ptr: *const Runtime = self;
            RUNTIME = r_ptr as usize;
        }
    }

    fn run(&mut self) {
        let mut can_next = true;
        while can_next {
            can_next = self.t_yield(); // thread 1 | thread 2 执行一遍就返回 base_thread 执行 yield 回来
        }
        println!("while finished");
        std::process::exit(0);
    }

    // 栈结束时候，重置可用状态
    fn t_return(&mut self) {
        if self.current != 0 {
            self.threads[self.current].state = State::Available; // 当前线程需要重新分配任务
            self.t_yield();
        }
    }

    #[inline(never)]
    fn t_yield(&mut self) -> bool {
        let mut pos = self.current;

        println!("current = {}", self.current);
        for i in 0..self.threads.len() {
            let thread = &self.threads[i];
            println!("the thread at index = {}, state = {:?}", i, thread.state);
        }
        println!("");

        // 找到 ready 的 thread
        while self.threads[pos].state != State::Ready {
            pos += 1;
            if pos == self.threads.len() {
                pos = 0;
            }
            if pos == self.current {
                return false;
            }
        }

        // 更新 old 为 ready, available -> running -> ready
        if self.threads[self.current].state != State::Available {
            self.threads[self.current].state = State::Ready;
        }

        self.threads[pos].state = State::Running; // 更新当前线程为 running 状态
        let old_pos = self.current; // 切换索引
        self.current = pos;

        let old: *mut ThreadContext = &mut self.threads[old_pos].ctx;
        let new: *const ThreadContext = &self.threads[pos].ctx;

        unsafe {
            switch(old, new);
        }

        self.threads.len() > 0
    }

    fn spawn(&mut self, f: fn()) {
        let available_thread = self
            .threads
            .iter_mut()
            .find(|t| t.state == State::Available)
            .expect("no available thread.");

        let size = available_thread.stack.len();

        unsafe {
            let s_ptr = available_thread.stack.as_mut_ptr().offset(size as isize);
            let s_aligned = (s_ptr as usize & !15) as *mut u8;

            // std::ptr::write(s_ptr.offset(-24) as *mut u64, guard as u64);
            std::ptr::write(s_aligned.offset(-8) as *mut u64, guard as u64);

            // std::ptr::write(s_ptr.offset(-32) as *mut u64, f as u64);
            std::ptr::write(s_aligned.offset(-16) as *mut u64, f as u64);

            // available_thread.ctx.rsp = s_ptr.offset(-32) as u64;
            available_thread.ctx.rsp = s_aligned.offset(-16) as u64;

            // println!("Thread {} stack setup:", self.threads.len());
            // println!("  Function at: {:p}", f as *const ());
            // println!("  Guard at: {:p}", guard as *const ());
            // println!("  RSP set to: {:p}", s_ptr.offset(-16));
        }

        available_thread.state = State::Ready;
    }

    fn spawnf<F: FnOnce() + 'static>(f: F) {
        unsafe {
            let rt_ptr = RUNTIME as *mut Runtime;

            let available_thread = (*rt_ptr)
                .threads
                .iter_mut()
                .find(|t| t.state == State::Available)
                .expect("no available thread.");
            available_thread.task = Some(Box::new(f));

            let size = available_thread.stack.len();

            let s_ptr = available_thread.stack.as_mut_ptr().offset(size as isize);
            let s_aligned = (s_ptr as usize & !15) as *mut u8;

            available_thread.ctx.thread_ptr = available_thread as *const Thread as u64; // set thread pointer address
            std::ptr::write(s_aligned.offset(-8) as *mut u64, guard as u64);
            std::ptr::write(s_aligned.offset(-16) as *mut u64, call as u64);
            available_thread.ctx.rsp = s_aligned.offset(-16) as u64;

            available_thread.state = State::Ready;
        }
    }
}

fn call(thread: u64) {
    let thread = unsafe { &mut *(thread as *mut Thread) };

    if let Some(f) = thread.task.take() {
        f();
    }
}

fn guard() {
    unsafe {
        let rt_ptr = RUNTIME as *mut Runtime;
        let rt = &mut *rt_ptr;
        println!("thread {} finished", rt.threads[rt.current].id);
        rt.t_return();
    }
}

fn yield_thread() {
    unsafe {
        let rt_ptr = RUNTIME as *mut Runtime;
        (*rt_ptr).t_yield();
    }
}

unsafe extern "C" {
    unsafe fn switch(old_ctx: *mut ThreadContext, new_ctx: *const ThreadContext);
}

// 不使用这种方式，rust 函数会对汇编做处理
// 可能被优化掉或编译器插入清理逻辑（epilogue）
// #[naked]
// #[inline(never)]
// unsafe fn no_use_switch(old_ctx: *mut ThreadContext, new_ctx: *const ThreadContext) {
//     unsafe {
//         asm!(
//             "mov [rdi + 0x00], rsp",
//             "mov [rdi + 0x08], r15",
//             "mov [rdi + 0x10], r14",
//             "mov [rdi + 0x18], r13",
//             "mov [rdi + 0x20], r12",
//             "mov [rdi + 0x28], rbx",
//             "mov [rdi + 0x30], rbp",
//             "mov rsp, [rsi + 0x00]",
//             "mov r15, [rsi + 0x08]",
//             "mov r14, [rsi + 0x10]",
//             "mov r13, [rsi + 0x18]",
//             "mov r12, [rsi + 0x20]",
//             "mov rbx, [rsi + 0x28]",
//             "mov rbp, [rsi + 0x30]",
//             "ret",
//             in("rdi") old_ctx,
//             in("rsi") new_ctx
//         );
//     }
// }

fn main() {
    println!("runtime run.");
    let mut runtime = Runtime::new();
    runtime.init();

    Runtime::spawnf(|| {
        println!("thread 1 starting");
        let id = 1;
        for i in 0..10 {
            println!("thread: {} counter: {}", id, i);
            yield_thread();
        }
        println!("thread 1 finished");
    });

    Runtime::spawnf(|| {
        println!("thread 2 starting");
        let id = 2;
        for i in 0..15 {
            println!("thread: {} counter: {}", id, i);
            yield_thread();
        }
        println!("thread 2 finished");
    });

    runtime.run();
}
