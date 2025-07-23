// The contents of this file provide a stub to implement a new instruction format.
// The purpose of including this file is to improve documentation of instruction formats.

/*

/// This file contains the definition of the $instruction instruction.

/*
    The $instruction format is defined as:
    | $fields |
    with:

    $field as $x;
    $field as $y;
*/

use crate::{instruction::{information::ArgumentType, AssembleResult, ErrorKind, RawInstruction}, parse::parse::AstKind};

/// Describe the purpose of $instruction.
#[derive(Debug)]
pub struct $instruction {
    op_code: u32,
}

// Define how to pack to raw
impl From<$instruction> for RawInstruction {
    fn from($instruction: $instruction) -> Self {
        RawInstruction::new(
            (op_code << 26)
            | ($field)
        )
    }
}

// Define how to unpack from raw
impl From<RawInstruction> for $instruction {
    fn from(raw: RawInstruction) -> Self {
        Self {
            op_code: (raw.raw >> 26) & 0b11_1111,
            $field: (raw.raw >> $x) & $y,  
        }
    }
}

// Define how to map a set of parsed arguments to this struct
impl $instruction {
    pub fn assign_$istruction_arguments(
        arguments: Vec<AstKind>,
        args_to_use: &[ArgumentType]
    ) -> AssembleResult<Self> {
        let mut $field = 0;

        for(i, passed) in arguments.into_iter().enumerate() {
            match passed {
                ArgumentType::$field => $field = passed.$unpacker(),
                _ => unreachable!(),
            }
        }

        Ok(Self {
            op_code: 0, // Will be filled in by caller
            $field,
        }) 
    }
}


*/