use crate::instruction::instruction_table::INSTRUCTION_TABLE;

/// This function determines if a given string is a standard instruction.
pub fn is_standard_instruction(passed_ident: &str) -> bool {
    if INSTRUCTION_TABLE.get(passed_ident).is_some() {
        return true;
    } else {
        return false;
    };
}
