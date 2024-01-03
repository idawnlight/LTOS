# trap.S
# Trap handler and global context
# Steve Operating System
# Stephen Marz
# 24 February 2019

#include "macro.S"

    .section .text
    .global  m_trap_vector
# This must be aligned by 4 since the last two bits
# of the mtvec register do not contribute to the address
# of this vector.
    .align   4
m_trap_vector:
# All registers are volatile here, we need to save them
# before we do anything.
    csrrw    t6, mscratch, t6
# csrrw will atomically swap t6 into mscratch and the old
# value of mscratch into t6. This is nice because we just
# switched values and didn't destroy anything -- all atomically!
# in cpu.rs we have a structure of:
# 32 gp regs 0
# 32 fp regs 256
# SATP register 512
# Trap stack 520
# CPU HARTID 528
# We use t6 as the temporary register because it is the very
# bottom register (x31)
    .set     i, 1
    .rept    30
    save_gp  %i
    .set     i, i+1
    .endr

# Save the actual t6 register, which we swapped into
# mscratch
    mv       t5, t6
    csrr     t6, mscratch
    save_gp  31, t5

# Restore the kernel trap frame into mscratch
    csrw     mscratch, t5

# Get ready to go into Rust (trap.rs)
# We don't want to write into the user's stack or whomever
# messed with us here.
    csrr     a0, mepc
    csrr     a1, mtval
    csrr     a2, mcause
    csrr     a3, mhartid
    csrr     a4, mstatus
    mv       a5, t5
    ld       sp, 520(a5)
    call     m_trap

# When we get here, we've returned from m_trap, restore registers
# and return.
# m_trap will return the return address via a0.

    csrw     mepc, a0

# Now load the trap frame back into t6
    csrr     t6, mscratch

# Restore all GP registers
    .set     i, 1
    .rept    31
    load_gp  %i
    .set     i, i+1
    .endr

# Since we ran this loop 31 times starting with i = 1,
# the last one loaded t6 back to its original value.

    mret

#
# machine-mode timer interrupt.
#
.globl   timervec
.align   4
timervec:
    # start.c has set up the memory that mscratch points to:
    # scratch[0,8,16] : register save area.
    # scratch[32] : address of CLINT's MTIMECMP register.
    # scratch[40] : desired interval between interrupts.
    csrrw    a0, mscratch, a0
    sd       a1, 0(a0)
    sd       a2, 8(a0)
    sd       a3, 16(a0)

    # schedule the next timer interrupt
    # by adding interval to mtimecmp.
    ld       a1, 32(a0)       # CLINT_MTIMECMP(hart)
    ld       a2, 40(a0)       # interval
    ld       a3, 0(a1)
    add      a3, a3, a2
    sd       a3, 0(a1)

    # raise a supervisor software interrupt.
    li       a1, 2
    csrw     sip, a1

    ld       a3, 16(a0)
    ld       a2, 8(a0)
    ld       a1, 0(a0)
    csrrw    a0, mscratch, a0

    mret


    .global  make_syscall
make_syscall:
    ecall
    ret
