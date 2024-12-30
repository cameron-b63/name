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

use crate::definitions::structs::{LineComponent, PseudoInstruction};

/// Possible assemble error codes
pub enum AssembleError {}

// This file contains the struct definition and extracted functions used in the assembler_logic file. There was far too much inlined, so I have extracted it.

#[derive(Debug)]
pub struct Assembler {
    pub(crate) pseudo_instruction_table: HashMap<&'static str, &'static PseudoInstruction>,
    pub section_dot_text: Vec<u8>,
    pub section_dot_data: Vec<u8>,
    pub section_dot_rel: Vec<u8>,
    pub section_dot_line: Vec<u8>,
    pub symbol_table: Vec<Symbol>,
    pub(crate) equivalences: HashMap<String, String>,
    pub(crate) errors: Vec<String>,
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
            section_dot_rel: vec![],
            section_dot_line: vec![],
            symbol_table: vec![],
            equivalences: HashMap::new(),
            errors: vec![],
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

    /// Add a label to the symbol table with the corresponding value. If a double update was attempted, errors vector will be extended.
    pub(crate) fn add_label(&mut self, ident: &String, value: u32) {
        // If symbol exists but with placeholder, we'll just want to update it.
        let existing_symbol = self
            .symbol_table
            .iter_mut()
            .find(|sym| &sym.identifier == ident);

        match existing_symbol {
            Some(sym) => {
                if sym.value != 0 {
                    self.errors
                        .push(format!("[*] On line {}:", self.line_number));
                    self.errors.push(format!(
                        " - Duplicate symbol definition for {}",
                        ident.clone()
                    ));
                    return;
                } else {
                    sym.value = value;
                    return;
                }
            }
            None => {} // Fall through
        }

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
            value: value,
            size: 4,
            visibility: Visibility::Local,
            section: self.current_section.clone(),
        });

        println!("Inserted symbol {} at 0x{:x}", ident, self.current_address);

        self.most_recent_label = ident.clone();
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

    /// Attempt to assemble a parsed line. If successful, add bytes to section .text - else, extend errors and keep it pushing.
    pub fn handle_assemble_instruction(
        &mut self,
        info: &InstructionInformation,
        args: &Vec<LineComponent>,
    ) {
        let assembled_instruction_result = assemble_instruction(info, &args);

        match assembled_instruction_result {
            Ok(assembled_instruction) => match assembled_instruction {
                packed => {
                    self.section_dot_text
                        .extend_from_slice(&packed.to_be_bytes());

                    pretty_print_instruction(&self.current_address, &packed);
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

        // If a relocation entry needs to be added, work with it.
        if info.relocation_type.is_some() {
            // There can be only one.
            let symbol_ident = args
                .iter()
                .filter_map(|arg| match arg {
                    LineComponent::Identifier(identifier) => Some(identifier.clone()),
                    _ => None,
                })
                .collect();
            let symbol_offset: u32 = self.get_symbol_offset(symbol_ident);

            let new_bytes: Vec<u8> = RelocationEntry {
                r_offset: self.current_address - MIPS_TEXT_START_ADDR,
                r_sym: symbol_offset,
                r_type: info.relocation_type.unwrap().clone(),
            }
            .to_bytes();
            self.section_dot_rel.extend(new_bytes);
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

    pub fn get_symbol_offset(&mut self, ident: String) -> u32 {
        match self
            .symbol_table
            .iter()
            .position(|sym| sym.identifier == ident)
        {
            Some(idx) => return (idx as u32) + 1,
            None => {
                self.add_label(&ident, 0);
                return self.symbol_table.len() as u32;
            }
        };
    }
}
