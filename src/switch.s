# .global switch
# switch:
#     mov [rdi + 0x00], rsp
#     mov [rdi + 0x08], r15
#     mov [rdi + 0x10], r14
#     mov [rdi + 0x18], r13
#     mov [rdi + 0x20], r12
#     mov [rdi + 0x28], rbx
#     mov [rdi + 0x30], rbp
#     mov rsp, [rsi + 0x00]
#     mov r15, [rsi + 0x08]
#     mov r14, [rsi + 0x10]
#     mov r13, [rsi + 0x18]
#     mov r12, [rsi + 0x20]
#     mov rbx, [rsi + 0x28]
#     mov rbp, [rsi + 0x30]
#     ret

.global switch
switch:
    movq %rsp, 0x00(%rdi)
    movq %r15, 0x08(%rdi)
    movq %r14, 0x10(%rdi)
    movq %r13, 0x18(%rdi)
    movq %r12, 0x20(%rdi)
    movq %rbx, 0x28(%rdi)
    movq %rbp, 0x30(%rdi)
    movq 0x00(%rsi), %rsp
    movq 0x08(%rsi), %r15
    movq 0x10(%rsi), %r14
    movq 0x18(%rsi), %r13
    movq 0x20(%rsi), %r12
    movq 0x28(%rsi), %rbx
    movq 0x30(%rsi), %rbp
    retq