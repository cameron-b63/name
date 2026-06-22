use super::registers::Register;
/// The table contained in this file defines the details for every register NAME needs to use in Coprocessor 0.
/// This simplifies the Register and Select field representation,
/// as the MIPS standard has overloaded meanings for certain registers in certain contexts.
/// It is entirely based on information from this [specification](https://s3-eu-west-1.amazonaws.com/downloads-mips/documents/MD00090-2B-MIPS32PRA-AFP-06.02.pdf).
pub struct Cp0RegisterInformation {
    name: Register,
    register: usize,
    /* The _select field is for different interpretations of the same bit.
    It is largely unused, except for certain instructions.
    It is included for now. */
    _select: usize,
}

/// This helper function allows for quick translation from a Register to a usize.
pub fn to_register(reg: Register) -> usize {
    match CP0_REGISTER_INFO.iter().find(|info| info.name == reg) {
        Some(information) => information.register,
        None => panic!("Coprocessor 0 register {:?} was not implemented.", reg),
    }
}

/// Correspondence between a Register in Coprocessor0 and its register number.
pub const CP0_REGISTER_INFO: &[Cp0RegisterInformation] = &[
    Cp0RegisterInformation {
        name: Register::Status,
        register: 12,
        _select: 0,
    },
    Cp0RegisterInformation {
        name: Register::Cause,
        register: 13,
        _select: 0,
    },
    Cp0RegisterInformation {
        name: Register::EPC,
        register: 14,
        _select: 0,
    },
];
