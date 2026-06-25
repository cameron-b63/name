// This file contains a bunch of helpers for interacting with Coprocessor0.

use crate::{
    exception::{
        fields::{get_field_set, FieldInfo},
        register_set::{get_info, Cp0RegisterInformation},
        registers::Cp0Register,
    },
    structs::Coprocessor0,
};

pub fn get_field_info(reg: Cp0Register, field_name: &'static str) -> FieldInfo {
    let field_set = match get_field_set(reg) {
        Some(fs) => fs,
        None => panic!("Field {} does not exist, missing info", field_name),
    };

    match field_set.iter().find(|info| info.name == field_name) {
        Some(info) => info.clone(),
        None => panic!("Field {} does not exist", field_name),
    }
}

impl Coprocessor0 {
    pub fn get_register_value(&self, reg: Cp0Register) -> u32 {
        let info: Cp0RegisterInformation = get_info(reg);
        return self.registers[info.register as usize][info.select as usize];
    }

    pub fn get_field_value(&self, reg: Cp0Register, field_info: FieldInfo) -> u32 {
        let value = self.get_register_value(reg);

        let length = field_info.msb - field_info.lsb + 1;
        let mask = if length == 32 {
            u32::MAX
        } else {
            (1 << length) - 1
        };

        (value >> field_info.lsb) & mask
    }

    pub fn set_register_value(&mut self, reg: Cp0Register, value: u32) {
        let info: Cp0RegisterInformation = get_info(reg);
        self.registers[info.register as usize][info.select as usize] = value;
    }

    pub fn set_field_value(&mut self, reg: Cp0Register, field_info: FieldInfo, value: u32) {
        let original = self.get_register_value(reg);

        let length = field_info.msb - field_info.lsb + 1;
        let field_mask = if length == 32 {
            u32::MAX
        } else {
            (1 << length) - 1
        };

        let clear_mask = !(field_mask << field_info.lsb);
        let cleared_original = original & clear_mask;
        let shifted_field = (value & field_mask) << field_info.lsb;

        self.set_register_value(reg, cleared_original | shifted_field);
    }
}

// Other helpers
impl Coprocessor0 {
    /// Set the current KSU mode in Status (kernel, debug, etc.)
    pub fn set_current_mode(&mut self, value: u32) {
        let ksu_field_info = get_field_info(Cp0Register::Status, "KSU");
        self.set_field_value(Cp0Register::Status, ksu_field_info, value);
    }

    /// Get EXL in Status
    pub fn get_exception_level(&self) -> u32 {
        let exl_field_info = get_field_info(Cp0Register::Status, "EXL");
        self.get_field_value(Cp0Register::Status, exl_field_info)
    }

    /// Set the EXL field of Status
    pub fn set_exception_level(&mut self, value: u32) {
        let exl_field_info = get_field_info(Cp0Register::Status, "EXL");
        self.set_field_value(Cp0Register::Status, exl_field_info, value);
    }

    pub fn get_exc_code(&mut self) -> u32 {
        let exc_code_field_info = get_field_info(Cp0Register::Cause, "ExcCode");
        self.get_field_value(Cp0Register::Status, exc_code_field_info)
    }

    /// Set the ExcCode in Cause
    pub fn set_exc_code(&mut self, value: u32) {
        let exc_code_field_info = get_field_info(Cp0Register::Cause, "ExcCode");
        self.set_field_value(Cp0Register::Status, exc_code_field_info, value);
    }

    /// Get the SmartMIPS ASE status
    pub fn is_smart_mode_ase_set(&self) -> bool {
        let sm_field_info = get_field_info(Cp0Register::Config3, "SM");

        self.get_field_value(Cp0Register::Config3, sm_field_info) == 1
    }
}
