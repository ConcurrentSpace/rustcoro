const DEFAULT_STACK_SIZE = 1024 * 1024 * 2;
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

impl Runtime {

}

pub struct Runtime {
    threads: Vec<Thread>,
    current: usize,
}

pub fn run() {
    let mut runtime = Runtime::new();
    runtime.init();

    runtime.spawn(|| {
        yield_thread();
    });

    runtime.spawn(|| {
        yield_thread();
    });

    runtime.run();
}