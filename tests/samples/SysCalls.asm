# Definitions of various useful system calls for MIPS assembly.
# This file must be included using .include "SysCalls.asm" rather
# than being assembled as a separate module.
# Written by John Cole starting January 2, 2022
	.eqv	SysPrintInt 	1
	.eqv	SysPrintFloat 	2
	.eqv	SysPrintDouble	3
	.eqv	SysPrintString	4
	.eqv	SysReadInt	    5
	.eqv	SysReadFloat	6
	.eqv	SysReadDouble	7
	.eqv	SysReadString	8
	.eqv	SysAlloc		9
	.eqv	SysExit		    10
	.eqv	SysPrintChar	11
	.eqv	SysReadChar	    12
	.eqv	SysOpenFile	    13
	.eqv	SysReadFile	    14
	.eqv	SysWriteFile	15
	.eqv	SysCloseFile	16
	.eqv	SysExitValue	17
