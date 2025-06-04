const std = @import("std");

const STACK_SIZE: usize = 1024;

const ThreadContext = struct {
    rsp: u64,
};

fn hello() noreturn { // todo: - why can't use callconv(.C)
    std.debug.print("hello wake up on a new stack\n", .{});
    while (true) {}
}

extern fn gt_switch(new_ctx: *const ThreadContext) void;
comptime {
    asm (@embedFile("switch.S"));
}

pub fn main() !void {
    var ctx = ThreadContext{
        .rsp = 0,
    };

    std.debug.print("size = {d}\n", .{@sizeOf(u64)});

    const stack = try std.heap.page_allocator.alignedAlloc(u8, 16, STACK_SIZE);
    defer std.heap.page_allocator.free(stack);

    // 1. 获取栈底位置
    const stack_bottom = @intFromPtr(stack.ptr) + STACK_SIZE; // get stack bottom

    // // 2. 确保16字节对齐
    // const aligned_base = stack_bottom & ~@as(usize, 15);

    const sb_aligned = (stack_bottom - @sizeOf(u64)) & ~@as(usize, 15); // todo: - 16 vs 8

    @as(*u64, @ptrFromInt(sb_aligned)).* = @intFromPtr(&hello);

    ctx.rsp = sb_aligned;
    gt_switch(&ctx);

    std.debug.print("hello zig coro\n", .{});
}
