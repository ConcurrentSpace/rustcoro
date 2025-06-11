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
    movq %rsp, 0x00(%rdi)   # 保存当前栈指针
    movq %r15, 0x08(%rdi)   # 保存r15
    movq %r14, 0x10(%rdi)   # 保存r14
    movq %r13, 0x18(%rdi)   # 保存r13
    movq %r12, 0x20(%rdi)   # 保存r12
    movq %rbx, 0x28(%rdi)   # 保存rbx
    movq %rbp, 0x30(%rdi)   # 保存基指针
    movq 0x00(%rsi), %rsp   # 恢复新栈指针
    movq 0x08(%rsi), %r15   # 恢复r15
    movq 0x10(%rsi), %r14   # 恢复r14
    movq 0x18(%rsi), %r13   # 恢复r13
    movq 0x20(%rsi), %r12   # 恢复r12
    movq 0x28(%rsi), %rbx   # 恢复rbx
    movq 0x30(%rsi), %rbp   # 恢复基指针
    retq                    # 返回(隐含跳转)