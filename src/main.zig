const std = @import("std");

const STACK_SIZE: usize = 1024;

const ThreadContext = struct {
    rsp: u64,
};

fn hello() noreturn { // todo: - why can't use callconv(.C)
    // std.debug.print("hello wake up on a new stack\n", .{}); // todo: - this place use to much stack
    const stdout = std.io.getStdOut().writer();
    stdout.writeAll("hello wake up on a new stack\n") catch unreachable;
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

    // std.debug.print("size = {d}\n", .{@sizeOf(u64)});

    const stack = try std.heap.page_allocator.alignedAlloc(u8, 16, STACK_SIZE);
    defer std.heap.page_allocator.free(stack);

    std.debug.print("Stack info:\n", .{});
    std.debug.print("Total size: {d}\n", .{STACK_SIZE});
    std.debug.print("Alignment: {d}\n", .{@alignOf(@TypeOf(stack))});
    std.debug.print("Base address: 0x{x}\n", .{@intFromPtr(stack.ptr)});

    // 1. 获取栈底位置
    const stack_bottom = @intFromPtr(stack.ptr) + STACK_SIZE;

    // 2. 确保16字节对齐
    const sb_aligned = stack_bottom & ~@as(usize, 15);

    // 3. 预留返回地址空间（16字节）并保持对齐
    const rsp = sb_aligned - 16; // todo: - why 16

    // 4. 函数指针写入栈
    @as(*u64, @ptrFromInt(rsp)).* = @intFromPtr(&hello);

    // 5. 写入返回地址
    ctx.rsp = rsp; // todo: - how to jump hello

    gt_switch(&ctx);

    // std.debug.print("hello zig coro\n", .{});
}
