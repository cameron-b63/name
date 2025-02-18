# Compute the first twelve Fibonacci numbers and put in array, then print.
# This is an example from a Web site, commented and modified by John Cole.
#
      	.include "SysCalls.asm"
     	.data
fibs: 	.word   0 : 12        # "array" of 12 words to contain fib values
size: 	.word  12             # size of "array"
nl:     .asciiz "\n"          # newline character

# .globl main

# Start of code.
      	.text
	    la   	$t0, fibs        # load address of array
	    la   	$t5, size        # load address of size variable
      	lw   	$t5, 0($t5)      # load array size
      	li   	$t2, 1           # 1 is first and second Fib. number
      	sw   	$t2, 0($t0)      # F[0] = 1
      	sw   	$t2, 4($t0)      # F[1] = F[0] = 1
      	addi 	$t1, $t5, -2     # Counter for loop, will execute (size-2) times
#		lw      $t5, 0($0)		 # Messed up instruction for testing

# Loop for computing numbers.
compute: 	lw   	$t3, 0($t0)      # Get value from array F[n]
      	lw   	$t4, 4($t0)      # Get value from array F[n+1]
      	add  	$t2, $t3, $t4    # $t2 = F[n] + F[n+1]
      	sw   	$t2, 8($t0)      # Store F[n+2] = F[n] + F[n+1] in array
      	addi 	$t0, $t0, 4      # increment address of Fib. number source
      	addi 	$t1, $t1, -1     # decrement loop counter
      	bgtz 	$t1, compute     # repeat if not finished yet.
      	la   	$a0, fibs        # first argument for print (array)
      	add  	$a1, $zero, $t5  # second argument for print (size)
      	jal  	print            # call print routine.
      	li   	$v0, SysExit     # system call for exit
      	syscall                    # we are out of here.

#########  routine to print the numbers on one line.

      .data
space:.asciiz  " "          # space to insert between numbers
head: .asciiz  "The Fibonacci numbers are:\n"
      .text
print:	add  	$t0, $zero, $a0  # starting address of array
      	add  	$t1, $zero, $a1  # initialize loop counter to array size
      	la   	$a0, head        # load address of print heading
      	li   	$v0, SysPrintString
      	syscall                    # print heading
print5: 	lw   	$a0, 0($t0)      # load fibonacci number for syscall
      	li   	$v0, SysPrintInt # specify Print Integer service
      	syscall                    # print fibonacci number
      	la   	$a0, space       # load address of spacer for syscall
      	li   	$v0, SysPrintString           # specify Print String service
      	syscall                    # output string
      	addi 	$t0, $t0, 4      # increment address
     	addi 	$t1, $t1, -1     # decrement loop counter
      	bgtz 	$t1, print5      # repeat if not finished
		
		# Print newline character
		li      $v0, SysPrintString
		la      $a0, nl
		syscall

      	jr   	$ra              # return
