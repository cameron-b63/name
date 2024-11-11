use crate::constants::MIPS_ADDRESS_ALIGNMENT;
// use std::collections::HashMap;
use crate::debug::debug_utils::{Breakpoint, DebuggerState};
use crate::structs::{LineInfo, ProgramState};

impl Breakpoint {
    pub fn new(bp_num: usize, line_address: u32, lineinfo: &Vec<LineInfo>, program_state: &mut ProgramState) -> Self {
        Breakpoint {
            // bp_num,
            line_num: {
                match lineinfo
                    .iter()
                    .find(|&line| line.start_address == line_address)
                {
                    Some(line) => line.line_number,
                    None => {
                        panic!("Breakpoint not found in memory.");
                    }
                }
            },
            address: line_address,
            replaced_instruction: program_state.insert_breakpoint(line_address, bp_num).unwrap()
        }
    }
    // assembler::add_label is not the solution to male loneliness
}

impl DebuggerState {
    pub fn new() -> Self {
        DebuggerState {
            global_bp_num: 0,
            breakpoints: Vec::<Breakpoint>::new(),
            // replaced_instructions: Vec::<u32>::new(),
            global_list_loc: 5,
        }
    }

    /* These are all functions that only impact the debugger and not the state of the program. */

    /// "pb"
    pub fn print_all_breakpoints(&self) -> Result<(), String> {
        println!("BP_NUM: LINE_NUM");
        // for (_address, bp) in &self.breakpoints {
        // for bp in &self.breakpoints {
        for bp_num in 0..self.breakpoints.len(){
            println!("{:>6}: {}", bp_num, self.breakpoints[bp_num].line_num);
        }
        return Ok(())
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

    /// "b"
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
                "Reached maximum number of breakpoints - cannot add anymore."
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
        
        self.breakpoints.insert(
            self.global_bp_num as usize,
            Breakpoint::new(self.global_bp_num, line_address, lineinfo, program_state),
        );

        // find the next empty space in the breakpoint vector
        while let Some(_) = self.breakpoints.get(self.global_bp_num as usize){
            self.global_bp_num += 1;
        }
        // self.breakpoints.push(Breakpoint::new(self.global_bp_num, line_address, lineinfo));

        program_state.cpu.pc = program_state.cpu.pc - MIPS_ADDRESS_ALIGNMENT;

        println!(
            "Successfully added breakpoint {} at line {}.",
            self.global_bp_num, line_num
        );
        Ok(())
    }

    /// "del"
    pub fn remove_breakpoint(&mut self, db_args: &Vec<String>) -> Result<(), String> {
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

        let _removed_breakpoint = match self.breakpoints.get(bp_num) {
            Some(bp) => bp,
            None => {
                return Err(format!("Breakpoint {} not found.", bp_num))
            }
        };

        self.breakpoints.remove(bp_num);
        self.global_bp_num = bp_num;
        Ok(())
        // if memory.read_byte()

        // if let Some(kvpair) = self.breakpoints
        //     .iter()
        //     .find(|brpt| brpt..bp_num == bp_num)
        // {
        //     let removed_element = swag.remove(kvpair.0);
        //     println!("Removed {:?}", removed_element);
        //     self.global_bp_num -= 1;
        //     Ok(())
        // } else {
        //     Err(format!("Breakpoint with bp_num {} not found", bp_num))
        // }
    }
}