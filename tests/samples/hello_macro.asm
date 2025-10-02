# Hello, world!
    .include    "SysCalls.asm"
    .data
    .eqv sym 0
    .eqv osym sym + 4
OurBelovedString:
    .asciiz     "Hello, World!\n"
OurBelovedString2:
    .asciiz     "sorry for the inconvenience\n"

	.macro	sys (%func)
	li	$v0, %func
	syscall
	.end_macro

	.macro	sys_arg %func %addy
    la  $a0, %addy
	li	$v0, %func
	syscall
	.end_macro

.text
.globl
main:
    la  $a0, OurBelovedString
    sys(SysPrintString)

    sys_arg(SysPrintString, OurBelovedString2)

    sys SysExit
