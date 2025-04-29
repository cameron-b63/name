# Hello, world!
    .include    "SysCalls.asm"
    .data
    .eqv sym 0
    .eqv osym sym + 4
OurBelovedString:
    .asciiz     "Hello, World!\n"

.text
.globl
main:
    la  $a0, OurBelovedString
    li  $v0, SysPrintString
    syscall
    li  $v0, SysExit
    syscall
