const std = @import("std");

const STACK_SIZE = 1024 * 1024 * 2;

comptime {
    asm (@embedFile("switch2.s"));
}
extern fn switch_ctx(old_ctx: *StackContext, new_ctx: *StackContext) void;

const StackContext = struct {
    rsp: u64 = 0,
    r15: u64 = 0,
    r14: u64 = 0,
    r13: u64 = 0,
    r12: u64 = 0,
    rbx: u64 = 0,
    rbp: u64 = 0,
};

const State = enum {
    start,
    running,
    suspended,
    done,
};

const Coroutine = struct {
    const Self = @This();

    allocator: std.mem.Allocator = undefined,

    stack: []align(16) u8 = undefined,
    context: StackContext,
    state: State = .start,

    fn init(allocator: std.mem.Allocator, func_entry: anytype) !Self {
        const typeinfo = @typeInfo(@TypeOf(func_entry));

        if (typeinfo == .null) {
            // std.debug.print("func entry is null\n", .{});
            const context = StackContext{};
            return .{
                .context = context,
            };
        } else {
            const stack = try allocator.alignedAlloc(u8, 16, STACK_SIZE);

            const stack_bottom = @intFromPtr(stack.ptr) + STACK_SIZE;
            const sb_aligned = stack_bottom & ~@as(usize, 15);
            const rsp = sb_aligned - 16;
            @as(*u64, @ptrFromInt(rsp)).* = @intFromPtr(&func_entry); // todo: - need check func validate

            const context = StackContext{ .rsp = rsp };

            return .{
                .allocator = allocator,
                .stack = stack,
                .context = context,
            };
        }
    }

    fn deinit(self: *Self) void {
        self.allocator.free(self.stack);
    }

    fn resumeFrom(self: *Self, coro: *Coroutine) void {
        switch_ctx(&coro.context, &self.context);
    }
};

var base_coro: Coroutine = undefined;
var count_coro: Coroutine = undefined;
var count: i32 = 1;

fn addCount() void {
    std.debug.print("add count comes in \n", .{});
    count += 1;

    base_coro.resumeFrom(&count_coro);

    count += 1;

    base_coro.resumeFrom(&count_coro);

    count += 1;

    base_coro.resumeFrom(&count_coro);
}

test "simple counter suspend and resume coroutine" {
    const allocator = std.testing.allocator;

    base_coro = try Coroutine.init(allocator, null);
    defer base_coro.deinit();
    count_coro = try Coroutine.init(allocator, addCount);
    defer count_coro.deinit();

    try std.testing.expect(1 == count);

    count_coro.resumeFrom(&base_coro);

    try std.testing.expect(2 == count);

    count_coro.resumeFrom(&base_coro);

    try std.testing.expect(3 == count);

    count_coro.resumeFrom(&base_coro);

    try std.testing.expect(4 == count);

    std.debug.print("all finished\n", .{});
}

// test "suspend and resume" {
//     var x: usize = 0;

//     const a = coroFn;

//     const frame = xasync(coroFn, .{&x});
//     try std.testing.expectEqual(x, 2);

//     const res = xawait(frame);
//     try std.testing.expectEqual(res, 12);
// }

// Frame -> Signature & Coroutine -> StackContext

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
