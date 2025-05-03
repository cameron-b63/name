use std::{collections::HashMap, sync::LazyLock};

use super::{
    fp_instruction_set::FP_INSTRUCTION_SET, instruction_set::INSTRUCTION_SET, InstructionMeta,
};

// Construct the instruction table for looking up instructions
pub static INSTRUCTION_TABLE: LazyLock<HashMap<&'static str, InstructionMeta>> =
    LazyLock::new(|| {
        let mut map = HashMap::new();

        // pull in all integer‐type instructions
        for info in INSTRUCTION_SET.iter() {
            map.insert(info.mnemonic, InstructionMeta::Int(info));
        }

        // pull in all FP‐type instructions
        for info in FP_INSTRUCTION_SET.iter() {
            map.insert(info.mnemonic, InstructionMeta::Fp(info));
        }

        map
    });
