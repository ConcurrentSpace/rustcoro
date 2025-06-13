.global switch
.section .text
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
    movq 0x38(%rsi), %rdi   # thread pointer 设置到第一个参数
    retq                    # 返回(隐含跳转)