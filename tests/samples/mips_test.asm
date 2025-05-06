# This program uses all of the implemented instructions in NAME
# to test expected behavior across all three pieces of the pipeline.

    .include    "SysCalls.asm"
    .data
coolfloat:
    .float      -3.14

skipNotSkippedString:
    .asciiz     "Something was meant to be skipped, but wasn't skipped.\n"

testString:
    .asciiz     "Hello, World!"

    .text
main:
    addi        $t0, $t0, 1
    add         $t1, $t0, $t0
    addiu       $t2, $t1, 0xFFFF
    addu        $t3, $t2, $t2
    and         $t3, $t2, $t3
    andi        $t3, $t3, 0xCC
    beq         $t4, $zero, demo1

skip1:
    # This label should be skipped.
    la          $a0, skipNotSkippedString
    li          $v0, SysPrintString
    syscall

demo1:
    li          $t3, 1337
    bgtz        $t3, demo2

skip2:
    # This label should be skipped.
    la          $a0, skipNotSkippedString
    li          $v0, SysPrintString
    syscall

demo2:
    li          $t0, -2
    blez        $t0, demo3

skip3:
    # This label should be skipped.
    la          $a0, skipNotSkippedString
    li          $v0, SysPrintString
    syscall

demo3:
    bne         $t0, $zero, demo4

skip4:
    # This label should be skipped
    la          $a0, skipNotSkippedString
    li          $v0, SysPrintString
    syscall

demo4:
    j           demo5

skip5:
    # This label should be skipped
    la          $a0, skipNotSkippedString
    li          $v0, SysPrintString
    syscall

demo5:
    jal         demo6

skip6:
    # This label should be skipped
    la          $a0, skipNotSkippedString
    li          $v0, SysPrintString
    syscall

demo6:
    la          $a0, demo7
    jalr        $a0

skip7:
    # This label should be skipped
    la          $a0, skipNotSkippedString
    li          $v0, SysPrintString
    syscall

demo7:
    la          $a0, demo8
    jr          $a0

skip8:
    # This label should be skipped
    la          $a0, skipNotSkippedString
    li          $v0, SysPrintString
    syscall

demo8:
    # The jump/branch gauntlet is over.
    la          $t0, testString
    lb          $t1, 1($t0)     # "e"
    lui         $t2, 42
    lw          $t1, 4($t0)     # "o, Wo"
    nor         $t2, $t2, $t1
    nop
    nop
    nop
    or          $t1, $t2, $t0
    ori         $t5, $zero, 'C'
    sb          $t5, $t0        # "Cello, World!"
    la          $a0, testString
    li          $v0, SysPrintString
    syscall
    sll         $t5, $t5, 2
    slt         $t6, $zero, $t5
    slti        $t7, $zero, -18
    sltiu       $t8, $zero, -18
    sltu        $t6, $t5, $zero
    srl         $t5, $t5, 2
    # sub         $t0, $t0, $t0
    subu        $t7, $t7, $t0
    la          $t0, testString
    sw          $t7, $t0
    la          $a0, testString
    li          $v0, SysPrintString
    # syscall
    xor         $t7, $t7, $zero
    xori        $t1, $t1, 0b10101010

demo9:
    la          $t0, coolfloat
    lwc1        $f12, $t0
    li          $v0, SysPrintFloat
    syscall

    abs.s       $f12, $f12
    syscall

exit:
    li          $v0, SysExit
    syscall
