# trap.S
# Assembly-level trap handler.

.global asm_trap_vector
asm_trap_vector:
    # We get here when the CPU is interrupted
	# for any reason.

    # handle timer interrupt
    # add mtimecmp
    la   a1, 0x2004000
    ld   a2, 0(a1)
    li   a3, 2000000
    add  a2, a2, a3
    sd   a2, 0(a1)

    mret