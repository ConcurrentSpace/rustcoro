const std = @import("std");

// Frame -> Signature & Coroutine -> StackContext

const STACK_SIZE = 1024 * 1024 * 2;

comptime {
    asm (@embedFile("switch2.s"));
}
extern fn switch_ctx(old_ctx: *StackContext, new_ctx: *StackContext) void;

const StackContext = struct {
    rsp: usize = 0,
    r15: usize = 0,
    r14: usize = 0,
    r13: usize = 0,
    r12: usize = 0,
    rbx: usize = 0,
    rbp: usize = 0,
};

const State = enum {
    start,
    running,
    suspended,
    done,
};

const Coroutine = struct {
    const Self = @This();

    stack: []align(16) u8,
    context: StackContext,
    state: State = .start,

    // todo: - change this allocator inner
    // fn init(allocator: std.mem.Allocator, func: anytype) !Self {
    //     const context = StackContext{};
    //     const stack = try allocator.alignedAlloc(u8, 16, STACK_SIZE);

    //     return .{};
    // }
};

const Signature = struct {
    const Self = @This();

    func_type: type,
    args_type: type,

    fn init(comptime func: anytype, comptime args: anytype) Self {
        _ = args;
        const func_type = @TypeOf(func);
        return .{
            .func_type = func_type,
        };
    }
};

fn Frame(comptime func: anytype, comptime args: anytype) type {
    _ = func;
    _ = args;
    return struct {
        const Self = @This();

        coroutine: *Coroutine,
    };
}

threadlocal var runtime: Runtime = .{};
const Runtime = struct {};

fn xresume(frame: anytype) void {
    const coroutine = frame.coroutine;
    runtime.switchTo(coroutine);
}

fn xsuspend() void {}

fn xasync(func: anytype, args: anytype) Frame(func) {
    const frame = Frame(func, args).init(args);
    xresume(frame);
    return frame;
}

fn xawait(comptime frame: anytype) void {
    _ = frame;
}

// fn ReturnType(comptie frame_type: type) type {
//     return frame_type.returnType();
// }

fn coroFn(x: *usize) usize {
    x.* += 2;

    xsuspend();

    return x.* + 10;
}

fn action1() void {
    for (0..10) |index| {
        std.debug.print("action1 index = {}\n", .{index});
    }
}

fn action2() void {
    for (0..10) |index| {
        std.debug.print("action2 index = {}\n", .{index});
    }
}

var coroutine1: Coroutine = undefined;
var coroutine2: Coroutine = undefined;

fn init_ctx(ctx: *StackContext, func_entry: anytype) !void {
    const allocator = std.heap.page_allocator;
    const stack = try allocator.alignedAlloc(u8, 16, STACK_SIZE);
    // defer std.heap.page_allocator.free(stack);

    // 1. 获取栈底位置
    const stack_bottom = @intFromPtr(stack.ptr) + STACK_SIZE;

    // 2. 确保16字节对齐
    const sb_aligned = stack_bottom & ~@as(usize, 15);

    // 3. 预留返回地址空间（16字节）并保持对齐
    const rsp = sb_aligned - 16;

    // 4. 函数指针写入栈
    @as(*u64, @ptrFromInt(rsp)).* = @intFromPtr(&func_entry);

    // 5. 写入返回地址
    ctx.rsp = rsp;
}

test "switch-stack-context" {}

// test "suspend and resume" {
//     var x: usize = 0;

//     const a = coroFn;

//     const frame = xasync(coroFn, .{&x});
//     try std.testing.expectEqual(x, 2);

//     const res = xawait(frame);
//     try std.testing.expectEqual(res, 12);
// }

pub fn main() !void {
    // action1();
    // action1();

    const allocator = std.heap.page_allocator;
    const stack = try allocator.alignedAlloc(u8, 16, STACK_SIZE);
    defer std.heap.page_allocator.free(stack);

    var main_ctx = StackContext{};
    var ctx1 = StackContext{};
    // var ctx2 = StackContext{};

    // 1. 获取栈底位置
    const stack_bottom = @intFromPtr(stack.ptr) + STACK_SIZE;

    // 2. 确保16字节对齐
    const sb_aligned = stack_bottom & ~@as(usize, 15);

    // 3. 预留返回地址空间（16字节）并保持对齐
    const rsp = sb_aligned - 16;

    // 4. 函数指针写入栈
    @as(*u64, @ptrFromInt(rsp)).* = @intFromPtr(&action1);
    // 5. 写入返回地址
    ctx1.rsp = rsp;
    // ctx1.rbp = stack_bottom;

    switch_ctx(&main_ctx, &ctx1);

    while (true) {}
}
