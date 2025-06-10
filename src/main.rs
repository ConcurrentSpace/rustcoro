use std::arch::{asm};

const DEFAULT_STACK_SIZE: usize = 1024 * 1024 * 2;
const MAX_THREADS: usize = 4;
static mut RUNTIME: usize = 0;

#[derive(Debug, Default)]
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

#[derive(PartialEq, Eq, Debug)]
enum State {
    Available,
    Running,
    Ready,
}

struct Thread {
    stack: Vec<u8>,
    ctx: ThreadContext,
    state: State,
}

impl Thread {
    fn new() -> Self {
        Thread {
            stack: vec![0_u8; DEFAULT_STACK_SIZE],
            ctx: ThreadContext::default(),
            state: State::Available,
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
            stack: vec![0_u8; DEFAULT_STACK_SIZE],
            ctx: ThreadContext::default(),
            state: State::Running,
        };

        let mut threads = vec![base_thread];
        let mut avaliable_threads: Vec<Thread> = (1..MAX_THREADS).map(|_| Thread::new()).collect();
        threads.append(&mut avaliable_threads);

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
        while self.t_yield() { }
        println!("while finished");
        std::process::exit(0);
    }

    fn t_return(&mut self) {
        if self.current != 0 {
            self.threads[self.current].state = State::Available;
            self.t_yield();
        }
    }

    #[inline(never)]
    fn t_yield(&mut self) -> bool {
        let mut pos = self.current;

        while self.threads[pos].state != State::Ready {
            pos += 1;
            if pos == self.threads.len() {
                pos = 0;
            }
            if pos == self.current {
                return false;
            }
        }

        if self.threads[self.current].state != State::Available {
            self.threads[self.current].state = State::Ready;
        }

        self.threads[pos].state = State::Running;
        let old_pos = self.current;
        self.current = pos;

        unsafe {
            let old: *mut ThreadContext = &mut self.threads[old_pos].ctx;
            let new: *const ThreadContext = &self.threads[pos].ctx;
            // asm!("call switch", in("rdi") old, in("rsi") new, clobber_abi("C")); // call switch
            switch(old, new);
        }

        self.threads.len() > 0
    }

    fn spawn(&mut self, f: fn()) {
        let available = self
            .threads
            .iter_mut()
            .find(|t| t.state == State::Available)
            .expect("no available thread.");

        let size = available.stack.len();

        unsafe {
            let s_ptr = available.stack.as_mut_ptr().offset(size as isize);
            let s_ptr = (s_ptr as usize & !15) as *mut u8;
            std::ptr::write(s_ptr.offset(-16) as *mut u64, guard as u64);
            std::ptr::write(s_ptr.offset(-14) as *mut u64, skip as u64);
            std::ptr::write(s_ptr.offset(-32) as *mut u64, f as u64);
            available.ctx.rsp = s_ptr.offset(-32) as u64;
        }

        available.state = State::Ready;
    }
}

fn guard() {
    unsafe {
        let rt_ptr = RUNTIME as *mut Runtime;
        (*rt_ptr).t_return();
    }
}

extern "C" fn skip() {
    unsafe  {
        asm!("ret")
    }
}

fn yield_thread() {
    unsafe {
        let rt_ptr = RUNTIME as *mut Runtime;
        (*rt_ptr).t_yield();
    }
}

unsafe fn switch(old_ctx: *mut ThreadContext, new_ctx: *const ThreadContext) {
    unsafe  {
        asm!(
            "mov [rdi + 0x00], rsp",
            "mov [rdi + 0x08], r15",
            "mov [rdi + 0x10], r14",
            "mov [rdi + 0x18], r13",
            "mov [rdi + 0x20], r12",
            "mov [rdi + 0x28], rbx",
            "mov [rdi + 0x30], rbp",
            "mov rsp, [rsi + 0x00]",
            "mov r15, [rsi + 0x08]",
            "mov r14, [rsi + 0x10]",
            "mov r13, [rsi + 0x18]",
            "mov r12, [rsi + 0x20]",
            "mov rbx, [rsi + 0x28]",
            "mov rbp, [rsi + 0x30]",
            "ret",
            in("rdi") old_ctx,
            in("rsi") new_ctx
        );
    }
}

// todo: - add save format

fn main() {
    println!("runtime run.");
    let mut runtime = Runtime::new();
    runtime.init();

    runtime.spawn(|| {
        println!("thread 1 starting");
        let id = 1;
        for i in 0..10 {
            println!("thread: {} counter: {}", id, i);
            yield_thread();
        }
        println!("thread 1 finished");
    });

    runtime.spawn(|| {
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