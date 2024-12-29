use name_core::{
    constants::REGISTERS,
    instruction::{
        information::{ArgumentType, InstructionInformation},
        instruction_set::INSTRUCTION_TABLE,
    },
    structs::Symbol,
};

use crate::definitions::{
    pseudo_instructions::PSEUDO_INSTRUCTION_SET,
    structs::{LineComponent, PseudoInstruction},
};

use std::collections::HashMap;

use super::assembler::Assembler;

// Helper function for assemble_instruction for use when multiple argument configurations are available.
// Checks argument configuration against what was passed.
// Returns a boolean value representing whether the expected fields matched or not.
pub fn arg_configuration_is_ok(
    passed_args: &Vec<LineComponent>,
    expected_args: &[ArgumentType],
) -> bool {
    if passed_args.len() != expected_args.len() {
        return false;
    }

    for (passed, expected) in passed_args.iter().zip(expected_args.iter()) {
        match (passed, expected) {
            (LineComponent::Register(_), ArgumentType::Rd)
            | (LineComponent::Register(_), ArgumentType::Rs)
            | (LineComponent::Register(_), ArgumentType::Rt)
            | (LineComponent::Immediate(_), ArgumentType::Immediate)
            | (LineComponent::Identifier(_), ArgumentType::Identifier)
            | (LineComponent::Identifier(_), ArgumentType::BranchLabel) => {}
            _ => return false,
        }
    }

    return true;
}

// Oft-used map operation nobody would want to repeat. Turns a symbol table entry into its address.
pub fn translate_identifier_to_address(
    identifier: &String,
    symbol_table: &Vec<Symbol>,
) -> Option<u32> {
    symbol_table
        .iter()
        .find(|symbol| symbol.identifier == identifier.clone())
        .map(|symbol| symbol.value)
}

// Parse a register string like "$t0" or "$3" to u32 for packing.
pub fn parse_register_to_u32(register: &String) -> Result<u32, String> {
    // Check the early exit
    if !register.starts_with("$") {
        return Err("Register parse failed.".to_string());
    }

    // First, try a simple lookup on the REGISTERS constant.
    if let Some(index) = REGISTERS.iter().position(|&x| x == register) {
        return Ok(index as u32);
    } else if let Ok(attempted_direct_parse) =
        register.chars().skip(1).collect::<String>().parse::<u32>()
    {
        // This line looks like wizard stuff but really I'm just removing the first char from the string by
        // using an iterator, skipping an item, and collecting everything else back together
        // This is for registers given like '$0' and '$3'
        return Ok(attempted_direct_parse);
    } else {
        return Err("Register parse failed".to_string());
    }
}

pub fn generate_pseudo_instruction_hashmap() -> HashMap<&'static str, &'static PseudoInstruction> {
    let mut hashmap: HashMap<&'static str, &'static PseudoInstruction> = HashMap::new();

    for pseudo in PSEUDO_INSTRUCTION_SET {
        hashmap.insert(pseudo.mnemonic, &pseudo);
    }

    hashmap
}

pub fn reverse_format_instruction(
    info: &InstructionInformation,
    args: &Vec<LineComponent>,
) -> String {
    // Prepare the mnemonic
    let mnemonic = &info.mnemonic;

    // Construct the operands string
    let operands: Vec<String> = args.iter().map(|arg| arg.to_string()).collect();
    let operands_str = operands.join(", ");

    // Define column positions
    let mnemonic_column = 9; // Column 10
    let operands_column = 17; // Column 18

    // Start building the instruction string
    let mut instruction = String::new();

    // Insert mnemonic at the correct column
    if mnemonic_column > 0 {
        instruction.push_str(&" ".repeat(mnemonic_column));
    }
    instruction.push_str(mnemonic);

    // Calculate spaces needed to align operands to the correct column
    let current_length = instruction.len();
    if operands_column > current_length {
        instruction.push_str(&" ".repeat(operands_column - current_length));
    }
    instruction.push_str(&operands_str);

    instruction
}

/*
Pretty print an instruction in the format:

 - 0xDEADBEEF
 - 0b11011110101011011011111011101111

*/
pub fn pretty_print_instruction(addr: &u32, packed: &u32) {
    println!(" - @0x{:08x}:", addr);
    println!("   - 0x{:08x}", packed);
    println!("   - 0b{:032b}", packed);
    println!();
}

pub fn search_mnemonic(
    mnemonic: String,
    environment: &mut Assembler,
) -> (
    Option<&'static InstructionInformation>,
    Option<&'static PseudoInstruction>,
) {
    // There are fewer pseudoinstruction mnemonics, and instructions like `li` and `la` are used incredibly often.
    // Therefore, search should happen for them first.
    // This is kind of an over-optimization but low-hanging fruit is low-hanging fruit.

    let mut instruction_information: Option<&'static InstructionInformation> = None;
    let mut pseudo_instruction_information: Option<&'static PseudoInstruction> = None;

    let retrieved_pseudo_instruction_option: Option<&'static PseudoInstruction> = environment
        .pseudo_instruction_table
        .get(mnemonic.as_str())
        .copied();
    match retrieved_pseudo_instruction_option {
        Some(pseudo_instruction_info) => {
            pseudo_instruction_information = Some(pseudo_instruction_info);
            // return (instruction_information, pseudo_instruction_information);
        }
        None => {
            // Do nothing. It's likely that it's an instruction instead.
        }
    }

    // Simple early exit - no match exists for both info types
    if pseudo_instruction_information.is_some() {
        return (instruction_information, pseudo_instruction_information);
    }

    let retrieved_instruction_option: Option<&InstructionInformation> =
        INSTRUCTION_TABLE.get(mnemonic.as_str()).map(|x| &**x);

    match retrieved_instruction_option {
        Some(retrieved_instruction_information) => {
            instruction_information = Some(retrieved_instruction_information);
        }
        None => {
            environment.errors.push(format!(
                "[*] On line {}{}:",
                environment.line_prefix, environment.line_number
            ));
            environment.errors.push(format!(" - Instruction \"{}\" not recognized. If this is a valid MIPS instruction, consider opening a pull request at https://github.com/cameron-b63/name.", mnemonic));
            instruction_information = None;
        }
    }

    (instruction_information, pseudo_instruction_information)
}
