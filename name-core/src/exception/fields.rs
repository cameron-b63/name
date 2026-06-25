use crate::exception::registers::Cp0Register;

// This file contains the enum definitions for each field of each coprocessor0 register.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FieldInfo {
    pub name: &'static str,
    pub msb: u8,
    pub lsb: u8,
    pub default_value: Option<u32>,
}

pub fn get_field_set(register: Cp0Register) -> Option<&'static [FieldInfo]> {
    match register {
        Cp0Register::Status => Some(STATUS_FIELDS),
        Cp0Register::Cause => Some(CAUSE_FIELDS),
        Cp0Register::Config3 => Some(CONFIG3_FIELDS),
        _ => None,
    }
}

/// Status (12,0)
pub static STATUS_FIELDS: &'static [FieldInfo] = &[
    FieldInfo {
        name: "CU3",
        msb: 31,
        lsb: 31,
        default_value: None,
    },
    FieldInfo {
        name: "CU2",
        msb: 30,
        lsb: 30,
        default_value: None,
    },
    FieldInfo {
        name: "CU1",
        msb: 29,
        lsb: 29,
        default_value: None,
    },
    FieldInfo {
        name: "CU0",
        msb: 28,
        lsb: 28,
        default_value: None,
    },
    FieldInfo {
        name: "RP",
        msb: 27,
        lsb: 27,
        default_value: None,
    },
    FieldInfo {
        name: "FR",
        msb: 26,
        lsb: 26,
        default_value: None,
    },
    FieldInfo {
        name: "RE",
        msb: 25,
        lsb: 25,
        default_value: None,
    },
    FieldInfo {
        name: "MX",
        msb: 24,
        lsb: 24,
        default_value: None,
    },
    FieldInfo {
        name: "PX",
        msb: 23,
        lsb: 23,
        default_value: None,
    },
    FieldInfo {
        name: "BEV",
        msb: 22,
        lsb: 22,
        default_value: None,
    },
    FieldInfo {
        name: "TS",
        msb: 21,
        lsb: 21,
        default_value: None,
    },
    FieldInfo {
        name: "SR",
        msb: 20,
        lsb: 20,
        default_value: None,
    },
    FieldInfo {
        name: "NMI",
        msb: 19,
        lsb: 19,
        default_value: None,
    },
    FieldInfo {
        name: "Impl",
        msb: 17,
        lsb: 16,
        default_value: None,
    },
    FieldInfo {
        name: "IM7",
        msb: 15,
        lsb: 15,
        default_value: None,
    },
    FieldInfo {
        name: "IM6",
        msb: 14,
        lsb: 14,
        default_value: None,
    },
    FieldInfo {
        name: "IM5",
        msb: 13,
        lsb: 13,
        default_value: None,
    },
    FieldInfo {
        name: "IM4",
        msb: 12,
        lsb: 12,
        default_value: None,
    },
    FieldInfo {
        name: "IM3",
        msb: 11,
        lsb: 11,
        default_value: None,
    },
    FieldInfo {
        name: "IM2",
        msb: 10,
        lsb: 10,
        default_value: None,
    },
    FieldInfo {
        name: "IM1",
        msb: 9,
        lsb: 9,
        default_value: None,
    },
    FieldInfo {
        name: "IM0",
        msb: 8,
        lsb: 8,
        default_value: None,
    },
    FieldInfo {
        name: "KX",
        msb: 7,
        lsb: 7,
        default_value: None,
    },
    FieldInfo {
        name: "SX",
        msb: 6,
        lsb: 6,
        default_value: None,
    },
    FieldInfo {
        name: "UX",
        msb: 5,
        lsb: 5,
        default_value: None,
    },
    FieldInfo {
        name: "UM",
        msb: 4,
        lsb: 4,
        default_value: None,
    },
    FieldInfo {
        name: "R0",
        msb: 3,
        lsb: 3,
        default_value: None,
    },
    FieldInfo {
        name: "ERL",
        msb: 2,
        lsb: 2,
        default_value: None,
    },
    FieldInfo {
        name: "EXL",
        msb: 1,
        lsb: 1,
        default_value: None,
    },
    FieldInfo {
        name: "IE",
        msb: 0,
        lsb: 0,
        default_value: None,
    },
    // Pseudo-fields
    FieldInfo {
        name: "KSU",
        msb: 4,
        lsb: 3,
        default_value: None,
    },
];

pub static CAUSE_FIELDS: &'static [FieldInfo] = &[FieldInfo {
    name: "ExcCode",
    msb: 6,
    lsb: 2,
    default_value: None,
}];

pub static CONFIG3_FIELDS: &'static [FieldInfo] = &[
    FieldInfo {
        name: "M",
        msb: 31,
        lsb: 31,
        default_value: None,
    },
    FieldInfo {
        name: "SM",
        msb: 1,
        lsb: 1,
        default_value: Some(0),
    },
    FieldInfo {
        name: "TL",
        msb: 0,
        lsb: 0,
        default_value: None,
    },
];
