use std::{collections::HashMap, sync::LazyLock};

use crate::instruction::information::InstructionInformation;

use super::{fp_instruction_set::FP_INSTRUCTION_SET, instruction_set::INSTRUCTION_SET};

// Construct the instruction table for looking up instructions
pub static INSTRUCTION_TABLE: LazyLock<HashMap<&'static str, &InstructionInformation>> =
    LazyLock::new(|| {
        let mut map = HashMap::new();

        // pull in all integer‐type instructions
        for info in INSTRUCTION_SET.iter() {
            map.insert(info.mnemonic, info);
        }

        // pull in all FP‐type instructions
        for info in FP_INSTRUCTION_SET.iter() {
            map.insert(info.mnemonic, info);
        }

        map
    });
