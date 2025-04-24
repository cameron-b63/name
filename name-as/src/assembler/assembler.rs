use std::{collections::HashMap, fmt, io};

use name_core::{
    constants::{MIPS_ADDRESS_ALIGNMENT, MIPS_DATA_START_ADDR, MIPS_TEXT_START_ADDR},
    elf_def::{RelocationEntry, STT_FUNC, STT_OBJECT},
    instruction::instruction_set::INSTRUCTION_TABLE,
    parse::{
        parse::{Ast, AstKind},
        span::Span,
    },
    structs::{LineInfo, Section, Symbol, Visibility},
};

use crate::assembler::assemble_instruction::assemble_instruction;

use crate::assembler::assembly_helpers::{
    generate_pseudo_instruction_hashmap, pretty_print_instruction,
};

use crate::definitions::structs::PseudoInstruction;

/// Possible assemble error codes
#[derive(Debug)]
pub enum ErrorKind {
    DuplicateSymbol(String),
    Io(io::Error),
    String(String),
    BadArguments,
    LabelOutsideOfSection,
    UnknownInstruction(String),
    InvalidShamt,
    InvalidArgument,
    ImmediateOverflow,
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorKind::DuplicateSymbol(str) => write!(f, "duplicate symbol: {}", str),
            ErrorKind::Io(err) => write!(f, "{:#?}", err),
            ErrorKind::String(s) => write!(f, "{}", s),
            ErrorKind::BadArguments => write!(f, "bad arguments"),
            ErrorKind::LabelOutsideOfSection => write!(f, "label outside of section"),
            ErrorKind::UnknownInstruction(s) => write!(f, "unkown instruction {}", s),
            ErrorKind::InvalidShamt => write!(f, "invalid shift amount"),
            ErrorKind::InvalidArgument => write!(f, "invalid argument"),
            ErrorKind::ImmediateOverflow => write!(f, "immediate overflow"),
        }
    }
}

pub type AssembleResult<T> = Result<T, ErrorKind>;
pub type AssembleError = Span<ErrorKind>;

// This file contains the struct definition and extracted functions used in the assembler_logic file. There was far too much inlined, so I have extracted it.

#[derive(Debug)]
pub struct Assembler {
    pub(crate) pseudo_instruction_table: HashMap<&'static str, &'static PseudoInstruction>,
    pub section_dot_text: Vec<u8>,
    pub section_dot_data: Vec<u8>,
    pub section_dot_rel: Vec<u8>,
    pub section_dot_line: Vec<u8>,
    pub symbol_table: Vec<Symbol>,
    pub(crate) equivalences: HashMap<String, u32>,
    pub(crate) current_section: Section,
    pub(crate) current_address: u32,
    pub(crate) text_address: u32,
    pub(crate) data_address: u32,
    pub(crate) line_number: usize,
    pub(crate) line_prefix: String,
    pub(crate) most_recent_label: String,
}

impl Assembler {
    // Initialize the assembler environment - default constructor.
    pub fn new() -> Self {
        Assembler {
            pseudo_instruction_table: generate_pseudo_instruction_hashmap(),
            section_dot_text: vec![],
            section_dot_data: vec![],
            section_dot_rel: vec![],
            section_dot_line: vec![],
            symbol_table: vec![],
            equivalences: HashMap::new(),
            // errors: vec![],
            current_section: Section::Null,
            current_address: 0,
            text_address: MIPS_TEXT_START_ADDR,
            data_address: MIPS_DATA_START_ADDR,
            line_number: 1,
            line_prefix: String::from(""),
            most_recent_label: String::from(""),
        }
    }

    /// Add a label to the symbol table with the corresponding value. If a double update was attempted, errors vector will be extended.
    pub(crate) fn add_label(&mut self, ident: &str, value: u32) -> Result<(), ErrorKind> {
        // If symbol exists but with placeholder, we'll just want to update it.
        let existing_symbol = self
            .symbol_table
            .iter_mut()
            .find(|sym| &sym.identifier == ident);

        match existing_symbol {
            Some(sym) => {
                if sym.value != 0 {
                    return Err(ErrorKind::DuplicateSymbol(ident.to_string()));
                } else {
                    sym.value = value;
                    return Ok(());
                }
            }
            None => {} // Fall through
        }

        let sym = Symbol {
            symbol_type: match self.current_section {
                Section::Null => {
                    return Err(ErrorKind::LabelOutsideOfSection);
                }
                Section::Text => STT_FUNC,
                Section::Data => STT_OBJECT,
            },
            identifier: ident.to_owned(),
            value: value,
            size: 4,
            visibility: Visibility::Local,
            section: self.current_section.clone(),
        };

        self.symbol_table.push(sym);

        println!("Inserted symbol {} at 0x{:x}", ident, self.current_address);

        self.most_recent_label = ident.to_string();

        Ok(())
    }

    /// Attempt to assemble a parsed line. If successful, add bytes to section .text - else, extend errors and keep it pushing.
    pub fn assemble_instruction(&mut self, instr: &str, args: Vec<AstKind>) -> AssembleResult<()> {
        let info = INSTRUCTION_TABLE
            .get(instr)
            .ok_or(ErrorKind::UnknownInstruction(instr.to_string()))?;

        if let Some(rel) = info.relocation_type {
            let symbol_ident = args
                .iter()
                .filter_map(|arg| match arg {
                    AstKind::Symbol(identifier) => Some(identifier.clone()),
                    _ => None,
                })
                .collect();

            let symbol_offset: u32 = self.get_symbol_offset(symbol_ident).unwrap();

            let new_bytes: Vec<u8> = RelocationEntry {
                r_offset: self.current_address - MIPS_TEXT_START_ADDR,
                r_sym: symbol_offset,
                r_type: info.relocation_type.unwrap().clone(),
            }
            .to_bytes();

            self.section_dot_rel.extend(new_bytes);
        }

        let packed = assemble_instruction(
            info,
            args.into_iter()
                .map(|arg| {
                    if let AstKind::Symbol(_) = arg {
                        AstKind::Immediate(0)
                    } else {
                        arg
                    }
                })
                .collect(),
        )?;
        self.section_dot_text
            .extend_from_slice(&packed.to_be_bytes());

        // pretty_print_instruction(&self.current_address, &packed);

        self.current_address += MIPS_ADDRESS_ALIGNMENT;
        Ok(())
    }

    pub fn assemble_pseduo_instruction(
        &mut self,
        pinstr: &str,
        args: Vec<AstKind>,
    ) -> AssembleResult<()> {
        let info = self
            .pseudo_instruction_table
            .get(pinstr)
            .ok_or(ErrorKind::UnknownInstruction(pinstr.to_string()))?;

        let instrs = (info.expand)(args).map_err(|e| ErrorKind::String(e))?;

        for instr in instrs {
            self.assemble_instruction(instr.0, instr.1)?;
        }

        Ok(())
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
    pub fn assemble_ast(&mut self, mut ast: AstKind) -> Result<(), ErrorKind> {
        match ast {
            AstKind::Label(s) => self.add_label(&s, self.current_address)?,
            AstKind::Asciiz(s) => self.assemble_asciiz(s),
            AstKind::Section(section) => match section {
                Section::Text => self.switch_to_text_section(),
                Section::Data => self.switch_to_data_section(),
                _ => panic!("other sections not implemented"),
            },
            AstKind::Instruction(instr, args) => {
                let ast_kinds = args.into_iter().map(|ast| ast.kind).collect();
                if self.pseudo_instruction_table.contains_key(instr.as_str()) {
                    self.assemble_pseduo_instruction(&instr, ast_kinds)?;
                } else {
                    self.assemble_instruction(&instr, ast_kinds)?
                }
            }
            AstKind::Immediate(_) => panic!(),
            AstKind::Symbol(_) => panic!(),
            AstKind::Register(_) => panic!(),
        }
        Ok(())
    }

    pub fn assemble(&mut self, asts: Vec<Ast>) -> Vec<Span<ErrorKind>> {
        let mut errs = Vec::new();
        for ast in asts {
            if let Err(err) = self.assemble_ast(ast.kind) {
                errs.push(Span {
                    src_span: ast.src_span,
                    kind: err,
                });
            }
        }
        errs
    }

    pub fn get_symbol_offset(&mut self, ident: String) -> Result<u32, ErrorKind> {
        match self
            .symbol_table
            .iter()
            .position(|sym| sym.identifier == ident)
        {
            Some(idx) => return Ok((idx as u32) + 1),
            None => {
                self.add_label(&ident, 0)?;
                return Ok(self.symbol_table.len() as u32);
            }
        };
    }
}
