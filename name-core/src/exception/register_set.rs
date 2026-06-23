use crate::exception::fields::{FieldInfo, STATUS_FIELDS};

use super::registers::Cp0Register;
/// The table contained in this file defines the details for every register NAME needs to use in Coprocessor 0.
/// This simplifies the Register and Select field representation,
/// as the MIPS standard has overloaded meanings for certain registers in certain contexts.
/// It is entirely based on information from this [specification](https://s3-eu-west-1.amazonaws.com/downloads-mips/documents/MD00090-2B-MIPS32PRA-AFP-06.02.pdf).
#[derive(Clone, Copy, Debug)]
pub struct Cp0RegisterInformation {
    pub name: Cp0Register,
    pub register: usize,
    pub select: usize,
    pub fields: Option<&'static [FieldInfo]>,
}

/// This helper function allows for quick translation from a Register (enum) to Register information.
pub fn get_info(reg: Cp0Register) -> Cp0RegisterInformation {
    match CP0_REGISTER_INFO.iter().find(|info| info.name == reg) {
        Some(information) => information.clone(),
        None => panic!("Coprocessor 0 register {:?} was not implemented.", reg),
    }
}

/// Correspondence between a Register in Coprocessor0 and its register number.
pub const CP0_REGISTER_INFO: &[Cp0RegisterInformation] = &[
    Cp0RegisterInformation {
        name: Cp0Register::Status,
        register: 12,
        select: 0,
        fields: Some(STATUS_FIELDS),
    },
    Cp0RegisterInformation {
        name: Cp0Register::Cause,
        register: 13,
        select: 0,
        fields: None,
    },
    Cp0RegisterInformation {
        name: Cp0Register::EPC,
        register: 14,
        select: 0,
        fields: None,
    },
];
