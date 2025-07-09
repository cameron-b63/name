const NUM_OF_REGISTERS: usize = 32; // Number of general purpose registers; there exist more.

// Base addresses for sections:
pub const MIPS_TEXT_START_ADDR: u32 = 0x00400000; // The address at which, by convention, MIPS begins the .text section
pub const MIPS_DATA_START_ADDR: u32 = 0x10010000; // The address at which, by convention, MIPS begins the .data section (I really typed this out again!)
pub const MIPS_HEAP_START_ADDR: u32 = 0x10040000; // Similarly, the heap for dynamic allocation growing upward
pub const MIPS_STACK_END_ADDR: u32 = 0x7ffffe00; // In like fashion, the stack, which grows downward
                                                 // pub const MIPS_KERNEL_START_ADDR: u32 = 0x90000000; // Kernel data (currently unused)
                                                 // pub const MIPS_MMIO_START_ADDR: u32 = 0xffff0000; // Memory-mapped I/O devices (currently unused)

// Key constants for other stuff:
pub const MIPS_ADDRESS_ALIGNMENT: u32 = 4; // MIPS is aligned by 4-byte word

pub const REGISTERS: [&'static str; NUM_OF_REGISTERS] = [
    "$zero", "$at", "$v0", "$v1", "$a0", "$a1", "$a2", "$a3", "$t0", "$t1", "$t2", "$t3", "$t4",
    "$t5", "$t6", "$t7", "$s0", "$s1", "$s2", "$s3", "$s4", "$s5", "$s6", "$s7", "$t8", "$t9",
    "$k0", "$k1", "$gp", "$sp", "$fp", "$ra",
];

/// The FPU control registers have a great deal of associated documentation.
/// I have enscapulated their defaults as a separate module.
pub mod fpu_control {
    /*!
     * Coprocessor 1 Floating Point Unit Control Registers:
     *
     * - FIR: Floating Point Implementation Register. Encodes what is and isn't supported.
     *
     * - FCSR: Floating Point Control and Status Register. Enscapsuulates FEXR, FENR, and FCCR content.
     *
     * - FEXR: Floating Point Exception Register. Alias to FCSR. Encodes the status and cause of exceptions in the FPU.
     *
     * - FENR: Floating Point Enables Register. Alias to FCSR. Encodes rounding mode and "Enables" information.
     *
     * - FCCR: Floating Point Condition Code Reigster. Alias to FCSR. Condition Codes for comparisons.
     */

    /**
    FIR (Floating Point Implementation Register);
    (FIR, CP1 Control Register 0)

     - (29) = 0; Dictates whether user-mode access of FRE is supported. No 64-bit FPU => 0.
     - (28) = 0; Dictates whether user can switch FR. (FREP = 0) => 0.
     - (27-24) = 0;  Impl-dependent => 0.
     - (23) = 1; Dictates whether 2008 Revision to IEEE 754 standard is supported. It is => 1.
     - (22) = 0; Indicates whether FPU registers are 64-bits wide. They are not => 0.
     - (21) = 0; Indicates whether long-word sign/magnitude is supported => 0.
     - (20) = 0; Indicates whether single word sign/magnitude is supported => 0.
     - (19) = 0; Indicates whether the MIPS 3-D ASE is supported => 0.
     - (18) = 0; Indicates whether the paired-single mode is supported => 0.
     - (17) = 1; Indicates whether double-word floating-point is supported => 1.
     - (16) = 1; Indicates whether single-precision floating-point is supported => 1.
     - (15-8) = 0; An implementation-specific ProcessorID field representing some of the
     supported capabilities of the FPU. There exists no standard table.
     This field is typically meant to identify an actual hardware component.
     As a reasonable placeholder => 0.
     - (7-0) = 0; This is an optional field not required for compliance => 0.

     References: [MIPS Specification](https://s3-eu-west-1.amazonaws.com/downloads-mips/documents/MD00082-2B-MIPS32INT-AFP-06.01.pdf),
     pp. 88;
    */
    pub const FIR_DEFAULT_VALUES: u32 = 0b00_0_0_0000_1_0_0_0_0_0_1_1_00000000_00000000;

    /**
    FCSR, Floating Point Control and Status Register;
    (FCSR, CP1 Control Register 31)

     - (31-25; 23) = 0; floating-point condition codes. Default to all 0's. Non-contiguous fields. Default => 0.
     - (24) = 1; floating-point operations WILL be flushed if subnormal.
     This is because it's unpredictable accross different hardware otherwise.\
     - (22-21) = 0; Implementation-dependent and unneeded => 0.
     - (20) = 0; Reserved => 0.
     - (19) = 1; Indicates if ABS.fmt and NEG.fmt instructions are compliant with IEEE 754-2008. They are => 1.
     - (18) = 1; Indicates if NAN handling follows legacy MIPS or IEEE 754-2008 recommendations. Sticking with IEEE => 1.
     - (17-12) = 0; Indicates the cause of an exception. Bit breakdown below:
     (17) = E (Unimplemented operation);
     (16) = V (Invalid operation);
     (15) = Z (Divide by zero);
     (14) = O (Overflow);
     (13) = U (Underflow);
     (12) = I (Inexact);
     - (11-7) = 0b11111; Enables bits: control whether or not a cause leads to an exception.
     (11) = V (Invalid operation) => 1;
     (10) = Z (Divide by zero) => 1;
     (9) = O (Overflow) => 1;
     (8) = U (Underflow) => 1;
     (7) = I (Inexact) => 1;
     - (6-2) = 0; Unused since all "enables" trigger exceptions => 0.
     - (1-0) = 0; Rounding mode is round to nearest (not round toward zero) => 0.
     */
    pub const FCSR_DEFAULT_VALUES: u32 = 0b0000000_1_0_00_0_1_1_000000_11111_00000_00;

    /// Constant index of FIR
    pub const FIR_INDEX: usize = 0;
    /// Constant index of FCSR
    pub const FCSR_INDEX: usize = 0;
}
