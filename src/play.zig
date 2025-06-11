const std = @import("std");

const StackContext = struct {
    rsp: usize,
    r15: usize,
    r14: usize,
    r13: usize,
    r12: usize,
    rbx: usize,
    rbp: usize,
};

const State = enum {
    new,
    running,
    suspended,
    finished,
};

const Coroutine = struct {
    stack: []align(16) u8,
    context: StackContext,
    state: State = .new,
};

const Signature = struct {
    const Self = @This();

    func_type: type,
    args_typeL type,

    fn init(comptime func: anytype, comptime args: anytype) Self {
        const func_type = @TypeOf(func);
        return .{
            .func_type = func_type,
        };
    }
};

fn Frame(comptime func: anytype, comptime args: anytype) type {
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

fn xawait(comptime frame: anytype) ReturnType {

}

fn ReturnType(comptie frame_type: type) type {
    return frame_type.returnType();
}

fn coroFn(x: *usize) usize {
    x.* += 2;

    xsuspend();

    return x.* + 10;
}

test "suspend and resume" {
    var x: usize = 0;

    const a = coroFn;

    const frame = xasync(coroFn, .{&x});
    try std.testing.expectEqual(x, 2);

    const res = xawait(frame);
    try std.testing.expectEqual(res, 12);
}

pub fn main() void {
    std.debug.print("hello coroutine\n", .{});
}
