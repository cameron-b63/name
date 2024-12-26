// This file contains relevant structs and methods to produce valid ET_REL and ET_EXEC files.
// It is based on https://refspecs.linuxfoundation.org/elf/elf.pdf

/*

   _____ ____  _   _  _____ _______       _   _ _______ _____
  / ____/ __ \| \ | |/ ____|__   __|/\   | \ | |__   __/ ____|
 | |   | |  | |  \| | (___    | |  /  \  |  \| |  | | | (___
 | |   | |  | | . ` |\___ \   | | / /\ \ | . ` |  | |  \___ \
 | |___| |__| | |\  |____) |  | |/ ____ \| |\  |  | |  ____) |
  \_____\____/|_| \_|_____/   |_/_/    \_\_| \_|  |_| |_____/



*/

// The following data definitions are required to construct ELF files.
// The meaning of each field is detailed in the appropriate struct.

// Section setup for ET_REL files
// These are the sections which should be present in each ET_REL constructed by the functions in this file.
pub const NUM_OF_SECTIONS: usize = 7; // This is e_shnum.
pub const SECTIONS: [&'static str; NUM_OF_SECTIONS] = [
    "", // Null (reserved) section
    ".text",
    ".data",
    ".symtab",
    ".strtab",
    ".line",
    ".shstrtab",
];

// Constants pertaining to MIPS conventions
pub const MIPS_ALIGNMENT: u32 = 0x1000; // The appropriate alignment for MIPS executables (from all my research)

// ELF File Header fields

// All of the following pertain to the e_ident field, and can be edited granularly below.
pub const EI_MAG: [u8; 4] = [0x7F, b'E', b'L', b'F']; // ELF magic bytes to signal that an ELF file follows
pub const EI_CLASS: u8 = 1; // Specify 32-bit format
pub const EI_DATA: u8 = 2; // Specify BIG endian
pub const EI_VERSION: u8 = 1; // Set original version for first iteration of constructing relocatable
pub const EI_OSABI: u8 = 0; // Specify System V IBA (specified in original ELF TIS)
pub const EI_ABIVERSION: u8 = 0; // Not needed
pub const EI_PAD: [u8; 7] = [0, 0, 0, 0, 0, 0, 0]; // Built-in padding
pub const EI_NIDENT: usize = 16; // Size of this header

// this is the full e_ident field (a complete constant. should never need to be changed).
pub const E_IDENT_DEFAULT: [u8; EI_NIDENT] = [
    EI_MAG[0],
    EI_MAG[1],
    EI_MAG[2],
    EI_MAG[3],
    EI_CLASS,
    EI_DATA,
    EI_VERSION,
    EI_OSABI,
    EI_ABIVERSION,
    EI_PAD[0],
    EI_PAD[1],
    EI_PAD[2],
    EI_PAD[3],
    EI_PAD[4],
    EI_PAD[5],
    EI_PAD[6],
];

// I am defining all feasible ET modes for later.
// const ET_NONE: u16 = 0;
pub const ET_REL: u16 = 1;
pub const ET_EXEC: u16 = 2;
// const ET_DYN: u16 = 3;

// all ELFs will first be constructed with e_type set to ET_REL. The linker handles any changes.
pub const E_TYPE_DEFAULT: u16 = ET_REL;

// the e_machine field value 8 represents the MIPS instruction set.
pub const E_MACHINE_DEFAULT: u16 = 8;

// versioning begins at 1.
pub const E_VERSION_DEFAULT: u32 = 1;

// for relocatable object files, e_entry is handled by the linker. For now, it will be set to 0.
pub const E_ENTRY_DEFAULT: u32 = 0;

// the e_phoff field can be known ahead of time, since the program header follows the ELF header by convention.
// it is simply the size of the elf header in bytes, as is already specified to be 52 for a 32-bit executable.
pub const E_PHOFF_DEFAULT: u32 = 52;

// by convention, the section header follows all sections and any other headers. its value cannot be known ahead of time.
// thus, the e_shoff field must be filled in after the full module has been assembled.

// for MIPS, the MIPS ELF specification lays out the e_flags field as follows (irrelevant fields commented out but retained for completeness):
pub const EF_MIPS_NONREORDER: u32 = 0x00000001; // No reordering of code to be done by assembler (better for education)
                                                // const EF_MIPS_PIC: u32 = 0x00000002;             // Contains position-independent code
                                                // const EF_MIPS_CPIC: u32 = 0x00000004;            // The PIC present follows standard conventions for calling PIC
                                                // const EF_MIPS_UCODE: u32 = 0x00000010;           // This file contains UCODE (obsolete)
pub const EF_MIPS_ABI2: u32 = 0x00000020; // This file has an EI_CLASS of ELFCLASS32 (ours does)
                                          // const EF_MIPS_OPTIONS_FIRST: u32 = 0x00000080;   // The .MIPS.options section in this file contains descriptors for ld. (not us)
                                          // const EF_MIPS_ARCH_ASE: u32 = 0x0f000000;        // Architectural extensions below are present:
                                          // const EF_MIPS_ARCH_ASE_MDMX: u32 = 0x08000000;   // Uses MDMX multimedia extensions
                                          // const EF_MIPS_ARCH_ASE_M16: u32 = 0x04000000;    // Uses MIPS-16 ISA extensions (we implement some of these as pseudo-instructions)
pub const EF_MIPS_ARCH: u32 = 0x30000000; // Architecture of the code (mips version) version IV is implmented in NAME

// The bitwise-or combination of selected flags gives the proper e_flags.
pub const E_FLAGS_DEFAULT: u32 = EF_MIPS_ARCH | EF_MIPS_ABI2 | EF_MIPS_NONREORDER;

// As stated, the ELF header size is known to be 52 for 32-bit binaries.
pub const E_EHSIZE_DEFAULT: u16 = 52;

// Similarly, the program header entry size is known to be 32 for 32-bit binaries. It's from the way the struct is defined.
pub const E_PHENTSIZE_DEFAULT: u16 = 32;

// For our use case, the number of entries in the program header is known.
// Each object file we assemble prior to linking will have 1 entry for .text, and 1 entry for .data. (only loadable semgents)
pub const E_PHNUM_DEFAULT: u16 = 2;

// Just like the other sizes, e_shentsize is known because it's derived from the struct.
pub const E_SHENTSIZE_DEFAULT: u16 = 40;

// For our use case, e_shnum is known.
// Each object file we assemble needs all program header entries, along with 2 entries for .debug and .line (debug/lineinfo). (plus 1 for null)
pub const E_SHNUM_DEFAULT: u16 = NUM_OF_SECTIONS as u16;

// By convention, e_shstrndx is set to the last value in the section header. The first index is reserved
pub const E_SHSTRNDX_DEFAULT: u16 = E_SHNUM_DEFAULT - 1;

// Program header consts

// Program header types
// const PT_NULL: u32 = 0;
pub const PT_LOAD: u32 = 1;

// pt_flags
pub const PF_X: u32 = 0x1; // Execute
pub const PF_W: u32 = 0x2; // Write
pub const PF_R: u32 = 0x4; // Read

// Symbol table consts
pub const STT_OBJECT: u8 = 1;
pub const STT_FUNC: u8 = 2;
pub const SH_ENTSIZE_SYMTAB: u32 = 16;

// Section header consts

// section header type indicators
pub const SHT_NULL: u32 = 0;
pub const SHT_PROGBITS: u32 = 1;
pub const SHT_SYMTAB: u32 = 2;
pub const SHT_STRTAB: u32 = 3;

// sh_flags  (unused commented out):
pub const SHF_WRITE: u32 = 0x1; // writable
pub const SHF_ALLOC: u32 = 0x2; // occupies memory during construction
pub const SHF_EXECINSTR: u32 = 0x4; // executable
                                    // const SHF_MERGE: u32 = 0x10;         // might be merged
pub const SHF_STRINGS: u32 = 0x20; // contains null-term strings
                                   // const SHF_INFO_LINK: u32 = 0x40;     // sh_info contains SHT index
                                   // const SHF_LINK_ORDER: u32 = 0x80;    // preserve order after combining
                                   // const SHF_OS_NONCONFORMING: u32 = 0x100; // non-standard OS handling required
                                   // const SHF_GROUP: u32 = 0x200;        // section is a member of a group
                                   // const SHF_TLS: u32 = 0x400;          // section holds thread-local data

/*

   _____ _______ _____  _    _  _____ _______ _____
  / ____|__   __|  __ \| |  | |/ ____|__   __/ ____|
 | (___    | |  | |__) | |  | | |       | | | (___
  \___ \   | |  |  _  /| |  | | |       | |  \___ \
  ____) |  | |  | | \ \| |__| | |____   | |  ____) |
 |_____/   |_|  |_|  \_\\____/ \_____|  |_| |_____/



*/

// This struct for the ELF file header was derived from information found at https://en.wikipedia.org/wiki/Executable_and_Linkable_Format
// as well as information from the original TIS ELF specification.
#[repr(C)]
// Used to avoid aligment issues. Not sure it's necessary but honestly better safe than sorry in this case.
#[derive(Debug, Default, Clone, Copy)]
pub struct Elf32Header {
    pub e_ident: [u8; 16], // Contains { {EI_MAG0 .. EIMAG3}, EI_CLASS, EI_DATA, EI_VERSION, EI_OSABI, EI_ABIVERSION, EI_PAD}
    pub e_type: u16,       // Identifies object file type (ET_REL before linking, ET_EXEC after)
    pub e_machine: u16,    // Target ISA
    pub e_version: u32, // ELF version (will be 1 prior to linking, and incremented to 2 once linking is complete)
    pub e_entry: u32,   // Address of program entry point
    pub e_phoff: u32,   // Program header offset
    pub(crate) e_shoff: u32, // Section header offset
    pub e_flags: u32, // Flags are interpreted differently based on OS. For our use case, I don't yet know what they must be.
    pub e_ehsize: u16, // Size of ELF Header (changes based on 32/64b arch)
    pub e_phentsize: u16, // Size of program header entry
    pub e_phnum: u16, // Number of entries in program header
    pub e_shentsize: u16, // Size of section header entry
    pub e_shnum: u16, // Number of entries in header entry
    pub e_shstrndx: u16, // Index of section header containing section names (section header string index)
}

// This associated function serializes the struct to bytes. This is for writing to file.
impl Elf32Header {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = vec![];
        // Append each field to that byte vector
        bytes.extend_from_slice(&self.e_ident);
        bytes.extend_from_slice(&self.e_type.to_be_bytes());
        bytes.extend_from_slice(&self.e_machine.to_be_bytes());
        bytes.extend_from_slice(&self.e_version.to_be_bytes());
        bytes.extend_from_slice(&self.e_entry.to_be_bytes());
        bytes.extend_from_slice(&self.e_phoff.to_be_bytes());
        bytes.extend_from_slice(&self.e_shoff.to_be_bytes());
        bytes.extend_from_slice(&self.e_flags.to_be_bytes());
        bytes.extend_from_slice(&self.e_ehsize.to_be_bytes());
        bytes.extend_from_slice(&self.e_phentsize.to_be_bytes());
        bytes.extend_from_slice(&self.e_phnum.to_be_bytes());
        bytes.extend_from_slice(&self.e_shentsize.to_be_bytes());
        bytes.extend_from_slice(&self.e_shnum.to_be_bytes());
        bytes.extend_from_slice(&self.e_shstrndx.to_be_bytes());

        bytes
    }
}

// This struct was also derived from https://en.wikipedia.org/wiki/Executable_and_Linkable_Format
// It will be found at e_phoff, and have e_phnum entries, each of size e_phentsize. That's why we specified it prior.
#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct Elf32ProgramHeader {
    pub(crate) p_type: u32, // Indicates type. PT_LOAD signals a loadable segment (.text, .rodata, .data, etc.)
    pub(crate) p_offset: u32, // Offset of segment in the file image
    pub(crate) p_vaddr: u32, // Virtual address of segment in memory
    pub(crate) p_paddr: u32, // Reserved for physical address of segment in memory (likely not used in our use case)
    pub(crate) p_filesz: u32, // Size in bytes of the segment in the file image, can be 0 (but why?)
    pub(crate) p_memsz: u32, // Size in bytes of the segment in memory, can be 0 for non-loaded (PT_NULL) segments
    pub(crate) p_flags: u32, // Segment dependent flags: PF_R = read, PF_W = write, PF_X = execute. All three should NEVER be specified, though that's not enforced explicitly in my implmentation. WX is also generally not a good plan.
    pub(crate) p_align: u32, // 0 and 1 specify no alignment, but positive powers of 2 specify p_vaddr = p_offset % p_align
}

// Similarly serialize Elf32ProgramHeader to bytes for writing to file
impl Elf32ProgramHeader {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = vec![];
        // Append all fields to bytes vec
        bytes.extend_from_slice(&self.p_type.to_be_bytes()); // To Big-Endian bytes
        bytes.extend_from_slice(&self.p_offset.to_be_bytes());
        bytes.extend_from_slice(&self.p_vaddr.to_be_bytes());
        bytes.extend_from_slice(&self.p_paddr.to_be_bytes());
        bytes.extend_from_slice(&self.p_filesz.to_be_bytes());
        bytes.extend_from_slice(&self.p_memsz.to_be_bytes());
        bytes.extend_from_slice(&self.p_flags.to_be_bytes());
        bytes.extend_from_slice(&self.p_align.to_be_bytes());

        bytes
    }
}

// This struct was indeed also derived from https://en.wikipedia.org/wiki/Executable_and_Linkable_Format
// It will be found at e_shoff, and have e_shnum entries, each of size e_shentsize. the string representation of the name of each entry is found at e_shstrndx.
#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct Elf32SectionHeader {
    pub(crate) sh_name: u32, // Offset to .shstrtab section containing name of this section
    pub(crate) sh_type: u32, // Type of header. SHT_PROGBITS signals program data, SHT_SYMTAB the symbol table, etc.
    pub(crate) sh_flags: u32, // Identifies section attributes: see const definitions
    pub(crate) sh_addr: u32, // Vaddr of section in memory (for loaded sections)
    pub(crate) sh_offset: u32, // Offset of the section in file image
    pub(crate) sh_size: u32, // Size in bytes of the section in the file image
    pub(crate) sh_link: u32, // Section index of section
    pub(crate) sh_info: u32, // Extra info on section
    pub(crate) sh_addralign: u32, // Required alignment of section. Must be a power of two
    pub(crate) sh_entsize: u32, // Size in bytes of each entry for sections that contain fixed-size entries (think tables)
}

impl Elf32SectionHeader {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = vec![];
        // Append all fields to bytes vec
        bytes.extend_from_slice(&self.sh_name.to_be_bytes()); // To Big-Endian bytes
        bytes.extend_from_slice(&self.sh_type.to_be_bytes());
        bytes.extend_from_slice(&self.sh_flags.to_be_bytes());
        bytes.extend_from_slice(&self.sh_addr.to_be_bytes());
        bytes.extend_from_slice(&self.sh_offset.to_be_bytes());
        bytes.extend_from_slice(&self.sh_size.to_be_bytes());
        bytes.extend_from_slice(&self.sh_link.to_be_bytes());
        bytes.extend_from_slice(&self.sh_info.to_be_bytes());
        bytes.extend_from_slice(&self.sh_addralign.to_be_bytes());
        bytes.extend_from_slice(&self.sh_entsize.to_be_bytes());

        bytes
    }
}

#[repr(C)]
#[derive(Debug, Default, Clone)]
pub struct Elf32Sym {
    pub st_name: u32,
    pub st_value: u32,
    pub st_size: u32,
    pub st_info: u8,
    pub st_other: u8,
    pub st_shndx: u16,
}

impl Elf32Sym {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = vec![];

        bytes.extend_from_slice(&self.st_name.to_be_bytes());
        bytes.extend_from_slice(&self.st_value.to_be_bytes());
        bytes.extend_from_slice(&self.st_size.to_be_bytes());
        bytes.extend_from_slice(&[self.st_info]);
        bytes.extend_from_slice(&[self.st_other]);
        bytes.extend_from_slice(&self.st_shndx.to_be_bytes());

        bytes
    }
}

// To construct an ET_REL ELF file, we'll use the following struct:
#[repr(C)]
#[derive(Debug, Default, Clone)]
pub struct Elf {
    pub file_header: Elf32Header,
    pub program_header_table: Vec<Elf32ProgramHeader>,
    pub sections: Vec<Vec<u8>>,
    pub section_header_table: Vec<Elf32SectionHeader>,
}
