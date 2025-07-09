// This file is meant for coprocessor 0's exception handler.

// The implementation here was derived entirely from this document: https://s3-eu-west-1.amazonaws.com/downloads-mips/documents/MD00090-2B-MIPS32PRA-AFP-06.02.pdf

use crate::structs::LineInfo;

/// SourceContext contains the information from section .line for retrieving source context in exceptions.
#[derive(Debug)]
pub struct SourceContext {
    pub lineinfo: Vec<LineInfo>,
    pub source_filenames: Vec<String>,
}

impl SourceContext {
    pub fn new(lineinfo: Vec<LineInfo>, source_filenames: Vec<String>) -> Self {
        Self {
            lineinfo,
            source_filenames,
        }
    }
}

// This enum contains all the exceptions we could generate.
#[derive(Debug)]
pub enum ExceptionType {
    // Interrupt,
    // TlbMod,
    // TlbLoad,
    // TlbStore,
    AddressExceptionLoad,
    AddressExceptionStore,
    BusFetch,
    BusLoadStore,
    Syscall,
    Breakpoint,
    ReservedInstruction,
    CoprocessorUnusable,
    ArithmeticOverflow,
    Trap,
    // MsaFloatingPoint,
    FloatingPoint,
    // Implementation Dependent,
    // Implementation Dependent,
    // TlbReadInhibit,
    // TlbExecutionInhibit,
    // MsaDisabled,
    // Reserved,
    // Watch,
    // MCheck,
    // Thread,
    // DspDisabled,
    // VirtualizedGuestException,
    // Reserved,
    // Reserved,
    // CacheErr,
    // Reserved,
}

/// This impl block allows us to simply translate from ExceptionType to ExcCode later with `e as u32`
impl From<ExceptionType> for u32 {
    fn from(e: ExceptionType) -> u32 {
        match e {
            ExceptionType::AddressExceptionLoad => 0x04,
            ExceptionType::AddressExceptionStore => 0x05,
            ExceptionType::BusFetch => 0x06,
            ExceptionType::BusLoadStore => 0x07,
            ExceptionType::Syscall => 0x08,
            ExceptionType::Breakpoint => 0x09,
            ExceptionType::ReservedInstruction => 0x0a,
            ExceptionType::CoprocessorUnusable => 0x0b,
            ExceptionType::ArithmeticOverflow => 0x0c,
            ExceptionType::Trap => 0x0d,
            ExceptionType::FloatingPoint => 0x0f,
        }
    }
}

/// This impl block allows for converting the other way around (ExcCode to ExceptionType):
impl TryFrom<u32> for ExceptionType {
    type Error = String;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0x04 => Ok(ExceptionType::AddressExceptionLoad),
            0x05 => Ok(ExceptionType::AddressExceptionStore),
            0x06 => Ok(ExceptionType::BusFetch),
            0x07 => Ok(ExceptionType::BusLoadStore),
            0x08 => Ok(ExceptionType::Syscall),
            0x09 => Ok(ExceptionType::Breakpoint),
            0x0a => Ok(ExceptionType::ReservedInstruction),
            0x0b => Ok(ExceptionType::CoprocessorUnusable),
            0x0c => Ok(ExceptionType::ArithmeticOverflow),
            0x0d => Ok(ExceptionType::Trap),
            0x0f => Ok(ExceptionType::FloatingPoint),
            _ => Err(format!(
                "Failed to coerce ExcCode {} to ExceptionType.",
                value
            )),
        }
    }
}
