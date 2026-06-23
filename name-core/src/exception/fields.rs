use crate::exception::registers::Cp0Register;

// This file contains the enum definitions for each field of each coprocessor0 register.
/// This struct simply encodes the msb and lsb for any field.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FieldInfo {
    pub name: &'static str,
    pub msb: u8,
    pub lsb: u8,
}

pub fn get_field_set(register: Cp0Register) -> Option<&'static [FieldInfo]> {
    match register {
        Cp0Register::Status => Some(STATUS_FIELDS),
        _ => None,
    }
}

/// Status (12,0)
pub static STATUS_FIELDS: &'static [FieldInfo] = &[
    FieldInfo {
        name: "CU3",
        msb: 31,
        lsb: 31,
    },
    FieldInfo {
        name: "CU2",
        msb: 30,
        lsb: 30,
    },
    FieldInfo {
        name: "CU1",
        msb: 29,
        lsb: 29,
    },
    FieldInfo {
        name: "CU0",
        msb: 28,
        lsb: 28,
    },
    FieldInfo {
        name: "RP",
        msb: 27,
        lsb: 27,
    },
    FieldInfo {
        name: "FR",
        msb: 26,
        lsb: 26,
    },
    FieldInfo {
        name: "RE",
        msb: 25,
        lsb: 25,
    },
    FieldInfo {
        name: "MX",
        msb: 24,
        lsb: 24,
    },
    FieldInfo {
        name: "PX",
        msb: 23,
        lsb: 23,
    },
    FieldInfo {
        name: "BEV",
        msb: 22,
        lsb: 22,
    },
    FieldInfo {
        name: "TS",
        msb: 21,
        lsb: 21,
    },
    FieldInfo {
        name: "SR",
        msb: 20,
        lsb: 20,
    },
    FieldInfo {
        name: "NMI",
        msb: 19,
        lsb: 19,
    },
    FieldInfo {
        name: "Impl",
        msb: 17,
        lsb: 16,
    },
    FieldInfo {
        name: "IM7",
        msb: 15,
        lsb: 15,
    },
    FieldInfo {
        name: "IM6",
        msb: 14,
        lsb: 14,
    },
    FieldInfo {
        name: "IM5",
        msb: 13,
        lsb: 13,
    },
    FieldInfo {
        name: "IM4",
        msb: 12,
        lsb: 12,
    },
    FieldInfo {
        name: "IM3",
        msb: 11,
        lsb: 11,
    },
    FieldInfo {
        name: "IM2",
        msb: 10,
        lsb: 10,
    },
    FieldInfo {
        name: "IM1",
        msb: 9,
        lsb: 9,
    },
    FieldInfo {
        name: "IM0",
        msb: 8,
        lsb: 8,
    },
    FieldInfo {
        name: "KX",
        msb: 7,
        lsb: 7,
    },
    FieldInfo {
        name: "SX",
        msb: 6,
        lsb: 6,
    },
    FieldInfo {
        name: "UX",
        msb: 5,
        lsb: 5,
    },
    FieldInfo {
        name: "UM",
        msb: 4,
        lsb: 4,
    },
    FieldInfo {
        name: "R0",
        msb: 3,
        lsb: 3,
    },
    FieldInfo {
        name: "ERL",
        msb: 2,
        lsb: 2,
    },
    FieldInfo {
        name: "EXL",
        msb: 1,
        lsb: 1,
    },
    FieldInfo {
        name: "IE",
        msb: 0,
        lsb: 0,
    },
    // Pseudo-fields
    FieldInfo {
        name: "KSU",
        msb: 4,
        lsb: 3,
    },
];
