# Disable generation of compressed instructions.
    .option  norvc

# Define a .data section.
    .section .data

# Define a .text.init section.
    .section .text.init

# Execution starts here.
.global _start
.global _kernel_stack_start
.global kinit
_start:
	la 		a0, _bss_start
	la		a1, _bss_end
	bgeu	a0, a1, 2f
1:
	sd		zero, (a0)
	addi	a0, a0, 8
	bltu	a0, a1, 1b
2:
	# Allocate 64K stack for each hart
	la sp, _kernel_stack_start
	li a0, 0x10000
	csrr a1, mhartid
	addi a1, a1, 1
    mul a0, a0, a1
    add sp, sp, a0
    # jump to kinit()
    call kinit