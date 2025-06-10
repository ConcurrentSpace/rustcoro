.globl gt_switch
.global gt_switch
.section .text

gt_switch:
    # 存储第一个参数 new_ctx
    mov (%rdi), %rsp # 这里配合 build.rs 使用
    # mov rsp, [rdi] # 这里配合 global_asm 使用
    ret