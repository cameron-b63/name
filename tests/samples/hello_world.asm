# Hello, world!
# .include    "SysCalls.asm"
    .data

OurBelovedString:
    .asciiz     "Hello, World!\n"

    .text
    la  $a0, OurBelovedString
    li  $v0, SysPrintString
    syscall
    li  $v0, SysExit
    syscall
