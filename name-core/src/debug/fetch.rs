use crate::{
    exception::definitions::ExceptionType, instruction::RawInstruction, structs::ProgramState,
};

// Fetch the next instruction and check that it's in acceptable memory space
pub fn fetch(program_state: &mut ProgramState) -> RawInstruction {
    if !program_state
        .memory
        .allows_execution_of(program_state.cpu.pc)
    {
        program_state.set_exception(ExceptionType::AddressExceptionLoad);
    }

    let mut fetched_instruction: u32 = 0;
    let mut i = 0;

    while i < 4 {
        match program_state.memory.read_byte(program_state.cpu.pc + i) {
            Ok(b) => fetched_instruction |= (b as u32) << (24 - i * 8),
            // This error is pretty much impossible to get with normal use.
            // However, I still want to represent it as best as possible.
            // Update; I have somehow caused this to trigger. Not really sure how either.
            // I really wish I wrote things down when it happened.
            Err(_) => program_state.set_exception(ExceptionType::BusFetch),
        };

        i += 1;
    }

    RawInstruction::new(fetched_instruction)
}
