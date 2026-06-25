use crate::{
    exception::{fields::get_field_set, register_set::CP0_REGISTER_INFO},
    structs::Coprocessor0,
};

impl Coprocessor0 {
    pub fn new() -> Self {
        let mut new_cp0 = Coprocessor0 {
            registers: [[0; 8]; 32],
            debug_mode: false,
        };

        // Fill defined default values
        CP0_REGISTER_INFO
            .iter()
            .for_each(|ri| match get_field_set(ri.name) {
                Some(field_set) => {
                    let mut register_value: u32 = 0;
                    field_set.iter().for_each(|fi| match fi.default_value {
                        Some(default) => register_value = default << fi.lsb,
                        None => (),
                    });
                    new_cp0.registers[ri.register][ri.select] = register_value;
                }
                None => (),
            });

        // Return filled object
        new_cp0
    }
}
