use std::collections::HashMap;
use std::path::PathBuf;

use name_core::{
    constants::{MIPS_ADDRESS_ALIGNMENT, MIPS_DATA_START_ADDR, MIPS_TEXT_START_ADDR},
    elf_def::{STT_FUNC, STT_OBJECT},
    instruction::information::InstructionInformation,
    parse::parse::Ast,
    structs::{Section, Symbol, Visibility},
};

use crate::assembler::assemble_instruction::assemble_instruction;
use crate::assembler::assembly_helpers::{
    generate_pseudo_instruction_hashmap, pretty_print_instruction,
};

use crate::definitions::constants::BACKPATCH_PLACEHOLDER;
use crate::definitions::structs::{Backpatch, BackpatchType, LineComponent, PseudoInstruction};

/// Possible assemble error codes
pub enum AssembleError {}

// This file contains the struct definition and extracted functions used in the assembler_logic file. There was far too much inlined, so I have extracted it.

#[derive(Debug)]
pub struct Assembler {
    pub(crate) pseudo_instruction_table: HashMap<&'static str, &'static PseudoInstruction>,
    pub section_dot_text: Vec<u8>,
    pub section_dot_data: Vec<u8>,
    pub section_dot_line: Vec<u8>,
    pub symbol_table: Vec<Symbol>,
    pub(crate) equivalences: HashMap<String, String>,
    pub(crate) errors: Vec<String>,
    pub(crate) backpatches: Vec<Backpatch>,
    pub(crate) current_section: Section,
    pub(crate) current_address: u32,
    pub(crate) current_dir: PathBuf,
    pub(crate) text_address: u32,
    pub(crate) data_address: u32,
    pub(crate) line_number: usize,
    pub(crate) line_prefix: String,
    pub(crate) most_recent_label: String,
}

impl Assembler {
    // Initialize the assembler environment - default constructor.
    pub(crate) fn new() -> Self {
        Assembler {
            pseudo_instruction_table: generate_pseudo_instruction_hashmap(),
            section_dot_text: vec![],
            section_dot_data: vec![],
            section_dot_line: vec![],
            symbol_table: vec![],
            equivalences: HashMap::new(),
            errors: vec![],
            backpatches: vec![],
            current_section: Section::Null,
            current_address: 0,
            current_dir: PathBuf::new(),
            text_address: MIPS_TEXT_START_ADDR,
            data_address: MIPS_DATA_START_ADDR,
            line_number: 1,
            line_prefix: String::from(""),
            most_recent_label: String::from(""),
        }
    }

    // Check if a target symbol exists in the symbol table.
    // Returns a boolean representing if the symbol is present.
    pub(crate) fn symbol_exists(&self, symbol_identifier: &String) -> bool {
        let ident = symbol_identifier.clone();
        self.symbol_table
            .iter()
            .find(|symbol| symbol.identifier == ident)
            .is_some()
    }

    // Add a backpatch to the backpatches list. Used if a forward reference was detected to signal that it must be resolved later.
    pub(crate) fn add_backpatch(
        &mut self,
        instruction_info: &'static InstructionInformation,
        args: &Vec<LineComponent>,
        ident: String,
        backpatch_type: BackpatchType,
    ) {
        match backpatch_type {
            BackpatchType::Standard | BackpatchType::Upper => {
                self.backpatches.push(Backpatch {
                    instruction_info: instruction_info,
                    backpatch_type: backpatch_type,
                    arguments: args.clone(),
                    undiscovered_identifier: ident,
                    backpatch_address: self.current_address,
                    byte_offset: self.section_dot_text.len(),
                    line_number: self.line_number,
                });
            }
            BackpatchType::Lower => {
                self.backpatches.push(Backpatch {
                    instruction_info: instruction_info,
                    backpatch_type: backpatch_type,
                    arguments: args.clone(),
                    undiscovered_identifier: ident,
                    backpatch_address: self.current_address + MIPS_ADDRESS_ALIGNMENT,
                    byte_offset: self.section_dot_text.len() + MIPS_ADDRESS_ALIGNMENT as usize,
                    line_number: self.line_number,
                });
            }
        }
    }

    // Add a label to the symbol table.
    pub(crate) fn add_label(&mut self, ident: &String) {
        self.symbol_table.push(Symbol {
            symbol_type: match self.current_section {
                Section::Null => {
                    self.errors
                        .push(format!("[*] On line {}:", self.line_number));
                    self.errors
                        .push(" - Cannot declare label outside a section.".to_string());
                    0
                }
                Section::Text => STT_FUNC,
                Section::Data => STT_OBJECT,
            },
            identifier: ident.to_owned(),
            value: self.current_address,
            size: 4,
            visibility: Visibility::Local,
            section: self.current_section.clone(),
        });

        println!("Inserted symbol {} at 0x{:x}", ident, self.current_address);

        self.most_recent_label = ident.clone();
    }

    // Resolve all backpatches attached to a label. Used once a forward-reference label has been discovered and defined.
    // FIXME: Needs to be updated to work properly with the Upper/Lower vairants of the backpatch struct.
    pub(crate) fn resolve_backpatches(&mut self, ident: &String) {
        let label: String = ident.clone();

        loop {
            let backpatch_found = self
                .backpatches
                .iter()
                .find(|backpatch| backpatch.undiscovered_identifier == label);
            let backpatch: &Backpatch;

            if backpatch_found.is_none() {
                break;
            } else {
                backpatch = backpatch_found.unwrap();
            }

            let backpatch_address: u32 = backpatch.backpatch_address;

            let assembled_result: Result<Option<u32>, String>;
            match backpatch.backpatch_type {
                BackpatchType::Standard => {
                    assembled_result = assemble_instruction(
                        backpatch.instruction_info,
                        &backpatch.arguments,
                        &self.symbol_table,
                        &backpatch_address,
                    )
                }
                BackpatchType::Upper => {
                    assembled_result = Ok(Some(
                        // Get previously assembled instruction bytes from section .text
                        u32::from_be_bytes(self.section_dot_text[backpatch.byte_offset..backpatch.byte_offset+4].try_into().unwrap())
                        // OR in the newly found symbol's upper portion
                        | (self.symbol_table.iter().find(|symbol| symbol.identifier == label).unwrap().value >> 16),
                    ))
                }
                BackpatchType::Lower => {
                    assembled_result = Ok(Some(
                        // Get previously assembled instruction bytes from section .text
                        u32::from_be_bytes(self.section_dot_text[backpatch.byte_offset..backpatch.byte_offset+4].try_into().unwrap())
                        // OR in the newly found symbol's lower portion
                        | (self.symbol_table.iter().find(|symbol| symbol.identifier == label).unwrap().value & 0xFFFF),
                    ))
                }
            }

            match assembled_result {
                Ok(assembled_instruction) => match assembled_instruction {
                    Some(word) => {
                        let insert_offset = backpatch.byte_offset;
                        let bytes_to_insert = word.to_be_bytes();

                        self.section_dot_text.splice(
                            insert_offset..insert_offset + 4,
                            bytes_to_insert.iter().cloned(),
                        );

                        let label = backpatch.undiscovered_identifier.clone();
                        let line = backpatch.line_number.clone();
                        println!(" - Backpatch resolved for label {label} on line {line}:");
                        pretty_print_instruction(&backpatch.backpatch_address, &word);

                        let found_index = self
                            .backpatches
                            .iter()
                            .position(|bp| bp == backpatch)
                            .expect("Literally impossible to get this error.");

                        self.backpatches.remove(found_index);
                    }
                    None => {
                        unreachable!("Backpatch unable to be resolved. This indicates a stupidly difficult extension error that is likely not your fault unless you contribute to the source code.");
                    }
                },
                Err(e) => {
                    let found_index = self
                        .backpatches
                        .iter()
                        .position(|bp| bp == backpatch)
                        .expect("Literally impossible to get this error.");

                    self.errors.push(format!(
                        "[*] While resolving backpatch on line {}:",
                        backpatch.line_number
                    ));
                    self.errors.push(e);

                    self.backpatches.remove(found_index);
                }
            }
        }
    }

    // Expand a line. Try replacing all instances of equivalences.
    pub fn expand_line(&self, line: &str) -> String {
        let mut expanded_line = String::new();

        // Replace equivalences
        for token in line.split_whitespace() {
            if let Some(expansion) = self.equivalences.get(token) {
                expanded_line.push_str(expansion);
            } else {
                expanded_line.push_str(token);
            }

            expanded_line.push(' ');
        }

        expanded_line.trim_end().to_string()
    }

    // Attempt to assemble a parsed line. If successful, add bytes to section .text - else, extend errors and keep it pushing.
    pub fn handle_assemble_instruction(
        &mut self,
        info: &InstructionInformation,
        args: &Vec<LineComponent>,
    ) {
        let assembled_instruction_result =
            assemble_instruction(info, &args, &self.symbol_table, &self.current_address);

        match assembled_instruction_result {
            Ok(assembled_instruction) => match assembled_instruction {
                Some(packed) => {
                    self.section_dot_text
                        .extend_from_slice(&packed.to_be_bytes());

                    pretty_print_instruction(&self.current_address, &packed);
                }
                None => {
                    self.section_dot_text
                        .extend_from_slice(&BACKPATCH_PLACEHOLDER.to_be_bytes());

                    println!(" - Placeholder bytes appended to section .text.\n");
                }
            },
            Err(e) => {
                self.errors.push(format!(
                    "[*] On line {}{}:",
                    self.line_prefix, self.line_number
                ));
                self.errors.push(e);
            }
        }

        self.current_address += MIPS_ADDRESS_ALIGNMENT;
    }

    pub fn assemble_instruction(
        &mut self,
        info: &InstructionInformation,
        args: Vec<Ast>,
    ) -> Result<AssembleError, ()> {
        todo!()
    }

    pub fn assemble_asciiz(&mut self, s: String) {
        // turn string to asciiz
        let mut to_push: Vec<u8> = s
            // Escape sequences
            .replace(r"\n", "\n")
            .replace(r"\t", "\t")
            .replace(r"\\", "\\")
            .into_bytes();

        // add a null terminator
        to_push.push(b'\0');

        //  increment current address
        self.current_address += to_push.len() as u32;

        // add string to data section
        self.section_dot_data.extend(&to_push);

        // use the string to set the size of the most recent symbol in table
        // TODO: refactor
        match self
            .symbol_table
            .iter_mut()
            .find(|s| s.identifier == self.most_recent_label)
        {
            Some(res) => res.size = to_push.len() as u32,
            None => {}
        }
    }

    /// entry point for folding ast into the environment
    pub fn assemble(&mut self, ast: Ast) -> Result<(), AssembleError> {
        match ast {
            // individual ast nodes that can be folded into environment
            Ast::Label(s) => Ok(self.add_label(&s)),
            Ast::Include(s) => todo!(),
            Ast::Asciiz(s) => Ok(self.assemble_asciiz(s)),
            Ast::Section(section) => {
                self.current_section = section;
                Ok(())
            }
            Ast::Instruction(info, args) => todo!(),
            Ast::Root(nodes) => {
                // TODO! nodes??
                for node in nodes {
                    self.assemble(node)?;
                }

                Ok(())
            }

            // ast nodes that should be ohterwise consumed
            Ast::Immediate(s) => todo!(),
            Ast::Eqv(_, _) => panic!(),
            Ast::Symbol(s) => panic!(),
            Ast::BaseAddress(_, _) => panic!(),
            Ast::Register(s) => panic!(),
            Ast::LabelRef(s) => panic!(),
        }
    }
}
