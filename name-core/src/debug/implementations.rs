// use crate::constants::MIPS_ADDRESS_ALIGNMENT;
// use std::collections::HashMap;
use crate::debug::debug_utils::{Breakpoint, DebuggerState};
use crate::structs::{LineInfo, ProgramState};

impl Breakpoint {
    pub fn new(
        bp_num: usize,
        line_address: u32,
        lineinfo: &Vec<LineInfo>,
        program_state: &mut ProgramState,
    ) -> Result<Self, String> {
        let old_instr = match program_state.insert_breakpoint(line_address, bp_num) {
            Ok(instr) => instr,
            Err(_e) => {
                return Err(format!("Attempted breakpoint insertion at invalid address 0x{:#08x} - please stick to .text.", line_address))
            }
        };

        let bp = Breakpoint {
            // bp_num,
            line_num: {
                match lineinfo
                    .iter()
                    .find(|&line| line.start_address == line_address)
                {
                    Some(line) => line.line_number,
                    None => {
                        panic!(
                            "Breakpoint not found in memory. (Something has gone seriously wrong.)"
                        );
                    }
                }
            },
            address: line_address,
            replaced_instruction: old_instr,
            already_executed: false,
        };

        Ok(bp)
    }

    pub fn already_executed(&self) -> bool {
        return self.already_executed;
    }

    pub fn flip_execution_status(&mut self) {
        self.already_executed = !self.already_executed;
    }
}

impl DebuggerState {
    pub fn new() -> Self {
        DebuggerState {
            global_bp_num: 0,
            breakpoints: Vec::<Breakpoint>::new(),
            global_list_loc: 5,
        }
    }

    /* These are all functions that only impact the debugger and not the state of the program. */

    /// Prints all breakpoints that have been created. Invoked by "pb" in the CLI.
    pub fn print_all_breakpoints(&self) -> Result<(), String> {
        println!("BP_NUM: LINE_NUM");
        // for (_address, bp) in &self.breakpoints {
        // for bp in &self.breakpoints {
        for bp_num in 0..self.breakpoints.len() {
            println!("{:>6}: {}", bp_num, self.breakpoints[bp_num].line_num);
        }
        return Ok(());
    }

    // This method is used to shorten list_text.
    // It lists the lines that surround lnum. Right now, that's fixed to be
    // within the range of plus or minus 4, but we can surely add a flag
    // to change that, if it ever matters enough.
    pub fn list_lines(&mut self, lineinfo: &Vec<LineInfo>, mut lnum: usize) {
        if lnum == 0 {
            lnum = self.global_list_loc;
        }

        let begin = lnum.saturating_sub(5);
        let end = std::cmp::min(lnum.saturating_add(3), lineinfo.len() - 1);
        for i in begin..=end {
            // let arrow = self.pc_is_on_this_address(lineinfo[i].start_address,);
            println!(
                "{:>3} #{:08x}  {}",
                lineinfo[i].line_number, lineinfo[i].start_address, lineinfo[i].content
            );
        }

        // by default, bind the global list pointer (i.e. the line number that is selected when no args are provided)
        // to this current line number.
        // in a hypothetical future, we can add a flag to make this an option
        if lnum + 9 <= lineinfo.len() {
            self.global_list_loc = lnum + 9;
        } else {
            self.global_list_loc = 5;
        }
    }

    /// Adds a breakpoint at the given line number. Invoked by "b" in the CLI.
    pub fn add_breakpoint(
        &mut self,
        lineinfo: &Vec<LineInfo>,
        db_args: &Vec<String>,
        program_state: &mut ProgramState,
    ) -> Result<(), String> {
        if db_args.len() != 2 {
            return Err(format!(
                "b expects 1 argument, received {}",
                db_args.len() - 1
            ));
        }

        if self.breakpoints.len() > ((1 << 20) - 1) {
            return Err(format!(
                "Reached maximum number of breakpoints - cannot add any more."
            ));
        }

        // grab line number and do error handling as necessary
        let line_num: u32 = match db_args[1].parse() {
            Ok(ln) => ln,
            Err(_) => return Err("b takes 32-bit unsigned int as input".to_string()),
        };

        if line_num > lineinfo.len().try_into().unwrap() {
            // something like that
            return Err(format!("{} exceeds number of lines in program.", line_num));
        }

        // get the line address associated with the line number
        let line_address: u32 = match lineinfo.iter().find(|line| line.line_number == line_num) {
            Some(ln) => ln.start_address,
            None => {
                return Err(format!(
                    "Line number {} not found in memory. Something has gone very wrong.",
                    line_num
                ))
            }
        };

        let new_bp =
            match Breakpoint::new(self.global_bp_num, line_address, lineinfo, program_state) {
                Ok(bp) => bp,
                Err(e) => return Err(format!("{e}")),
            };

        self.breakpoints.insert(self.global_bp_num as usize, new_bp);

        // find the next empty space in the breakpoint vector
        while let Some(_) = self.breakpoints.get(self.global_bp_num as usize) {
            self.global_bp_num += 1;
        }

        // play around with this if you're getting weird pc related logic errors.
        // program_state.cpu.pc = program_state.cpu.pc - MIPS_ADDRESS_ALIGNMENT;

        println!(
            "Successfully added breakpoint {} at line {}.",
            self.global_bp_num, line_num
        );
        Ok(())
    }

    /// Zoinks a breakpoint. Invoked by "del" in the CLI.
    pub fn remove_breakpoint(
        &mut self,
        db_args: &Vec<String>,
        program_state: &mut ProgramState,
    ) -> Result<(), String> {
        if db_args.len() != 2 {
            return Err(format!(
                "del expects 1 argument, received {}",
                db_args.len() - 1
            ));
        }

        let bp_num: usize = match db_args[1].parse() {
            Ok(num) => num,
            Err(_) => {
                return Err("del takes an unsigned int as input".to_string());
            }
        };

        let removed_breakpoint = match self.breakpoints.get(bp_num) {
            Some(bp) => bp,
            None => return Err(format!("Breakpoint {} not found.", bp_num)),
        };

        // replace the instruction
        let mut i = 0;
        while i < 4 {
            // Shift/mask value to get correct byte
            let new_byte: u8 = ((removed_breakpoint.replaced_instruction >> (i * 8)) & 0xFF) as u8;
            // Write it to correct location
            match program_state
                .memory
                .set_byte(removed_breakpoint.address + (3 - i), new_byte)
            {
                Ok(_) => (),
                Err(e) => {
                    return Err(format!("{e}"));
                }
            }
            i += 1;
        }

        // remove the breakpoint from the universe of discourse
        self.breakpoints.remove(bp_num);

        // the index at bp_num is now empty. point our number there
        if self.global_bp_num > bp_num {
            self.global_bp_num = bp_num;
        }
        Ok(())
    }
}
