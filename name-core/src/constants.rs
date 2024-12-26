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
