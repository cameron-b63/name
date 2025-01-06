use std::{
    collections::HashMap,
    fs, io,
    path::{Path, PathBuf},
};

use name_core::{
    constants::{MIPS_ADDRESS_ALIGNMENT, MIPS_DATA_START_ADDR, MIPS_TEXT_START_ADDR},
    elf_def::{RelocationEntry, STT_FUNC, STT_OBJECT},
    instruction::{information::InstructionInformation, instruction_set::INSTRUCTION_TABLE},
    parse::{
        lexer::{Lexer, LexerError},
        parse::{Ast, ParseError, Parser},
    },
    structs::{LineInfo, Section, Symbol, Visibility},
};

use crate::assembler::assemble_instruction::assemble_instruction;

use crate::assembler::assembly_helpers::{
    generate_pseudo_instruction_hashmap, pretty_print_instruction,
};

use crate::definitions::structs::{LineComponent, PseudoInstruction};

/// Possible assemble error codes
#[derive(Debug)]
pub enum AssembleError<'a> {
    ParseError(ParseError<'a>),
    LexerError(LexerError<'a>),
    DuplicateSymbol(String),
    Io(io::Error),
    String(String),
    LabelOutsideOfSection,
}

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
    pub(crate) current_dir: PathBuf,
    pub(crate) text_address: u32,
    pub(crate) data_address: u32,
    pub(crate) line_number: usize,
    pub(crate) line_prefix: String,
    pub(crate) most_recent_label: String,
}

impl Assembler {
    // Initialize the assembler environment - default constructor.
    pub fn new(current_dir: PathBuf) -> Self {
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
            current_dir,
            text_address: MIPS_TEXT_START_ADDR,
            data_address: MIPS_DATA_START_ADDR,
            line_number: 1,
            line_prefix: String::from(""),
            most_recent_label: String::from(""),
        }
    }

    pub fn string_error(&mut self, err: String) {
        todo!()
    }

    /// Add a label to the symbol table with the corresponding value. If a double update was attempted, errors vector will be extended.
    pub(crate) fn add_label(&mut self, ident: &str, value: u32) -> Result<(), AssembleError> {
        // If symbol exists but with placeholder, we'll just want to update it.
        let existing_symbol = self
            .symbol_table
            .iter_mut()
            .find(|sym| &sym.identifier == ident);

        match existing_symbol {
            Some(sym) => {
                if sym.value != 0 {
                    return Err(AssembleError::DuplicateSymbol(ident.to_string()));
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
                    return Err(AssembleError::LabelOutsideOfSection);
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

    // Expand a line. Try replacing all instances of equivalences.
    // pub fn expand_line(&self, line: &str) -> String {
    //     let mut expanded_line = String::new();
    //
    //     // Replace equivalences
    //     for token in line.split_whitespace() {
    //         if let Some(expansion) = self.equivalences.get(token) {
    //             expanded_line.push_str(expansion);
    //         } else {
    //             expanded_line.push_str(token);
    //         }
    //
    //         expanded_line.push(' ');
    //     }
    //
    //     expanded_line.trim_end().to_string()
    // }

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
                self.string_error(format!(
                    "[*] On line {}{}:",
                    self.line_prefix, self.line_number
                ));
                // self.errors.push(AssembleError::String(e));
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

    pub fn assemble_instruction(&mut self, instr: &str, args: Vec<Ast>) {
        let info = INSTRUCTION_TABLE.get(instr).ok_or(()).copied();

        todo!("assemble instruction")
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

    // workhorse assemble a file, perform effects and report errors
    // returns false when there are errors
    pub fn assemble_file(&mut self, path: &Path) -> bool {
        // erros to report
        let mut errors = Vec::new();

        let content = fs::read_to_string(&path).unwrap_or_else(|e| {
            // report the io error
            errors.push(AssembleError::Io(e));
            // default to nothing
            "".into()
        });

        // lex the file contents
        let mut lexer = Lexer::new(&content);
        let (errs, toks) = lexer.lex();

        // report lex errors
        errors.extend(errs.into_iter().map(|err| AssembleError::LexerError(err)));

        // parsed lexed tokens into ast
        let mut parser = Parser::new(toks);
        let (perrs, ast) = parser.parse();

        // report parse erros
        errors.extend(perrs.into_iter().map(|err| AssembleError::ParseError(err)));

        // fold the ast into the environment
        self.assemble_ast(ast);

        // process line info
        for line in content.split('\n') {
            let start_address = match self.current_section {
                Section::Text => self.current_address,
                Section::Data => self.text_address,
                Section::Null => 0,
            };

            // Extend section .line to include the new line
            self.section_dot_line.extend(
                LineInfo {
                    content: line.to_string(),
                    line_number: self.line_number as u32,
                    start_address: match self.current_section {
                        Section::Text => start_address,
                        _ => 0,
                    },
                    end_address: match self.current_section {
                        Section::Text => self.current_address,
                        Section::Data => self.text_address,
                        _ => 0,
                    },
                }
                .to_bytes(),
            );

            self.line_number += 1;
        }

        let res = errors.is_empty();

        for error in errors {
            dbg!(error);
        }

        res
    }

    pub fn macro_expand(ident: &str, args: &[Ast]) -> Vec<Ast> {
        todo!("write this");
    }

    /// entry point for folding ast into the environment
    pub fn assemble_ast(&mut self, ast: Ast) -> Result<(), AssembleError> {
        match ast {
            // individual ast nodes that can be folded into environment
            Ast::Label(s) => self.add_label(&s, self.current_address)?,
            Ast::Include(s) => {
                let _ = self.assemble_file(self.current_dir.join(s).as_path());
            }
            Ast::Asciiz(s) => self.assemble_asciiz(s),
            Ast::Eqv(ident, value) => {
                let _ = self.equivalences.insert(ident, value);
            }
            Ast::Section(section) => match section {
                Section::Text => self.switch_to_text_section(),
                Section::Data => self.switch_to_data_section(),
                _ => panic!("other sections not implemented"),
            },
            Ast::Instruction(instr, args) => self.assemble_instruction(&instr, args),
            Ast::Root(entries) => {
                for entry in entries {
                    self.assemble_ast(entry);
                }
            }
            Ast::MacroDefintion(ident, args, body) => {
                todo!("store macro defintion");
            }
            Ast::Macro(ident, args) => {
                let vec_ast = self.macro_expand(ident, args);
                for ast in vec_ast {
                    self.assemble_ast(ast);
                }
            }
            // ast nodes that should be ohterwise consumed
            Ast::Immediate(_) => panic!(),
            Ast::Symbol(_) => panic!(),
            Ast::BaseAddress(_, _) => panic!(),
            Ast::Register(_) => panic!(),
        }
        Ok(())
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
