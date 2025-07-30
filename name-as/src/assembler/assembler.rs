use std::collections::HashMap;

use name_core::{
    constants::{MIPS_ADDRESS_ALIGNMENT, MIPS_DATA_START_ADDR, MIPS_TEXT_START_ADDR},
    elf_def::{RelocationEntry, STT_FUNC, STT_OBJECT},
    instruction::{
        pseudo_instruction_set::PseudoInstruction,
        {instruction_table::INSTRUCTION_TABLE, AssembleResult, ErrorKind, RawInstruction},
    },
    parse::{
        parse::{Ast, AstKind},
        span::Span,
    },
    structs::{Section, Symbol, Visibility},
};

use crate::assembler::assemble_instruction::assemble_instruction;

use crate::assembler::assembly_helpers::generate_pseudo_instruction_hashmap;

// This file contains the struct definition and extracted functions used in the assembler_logic file. There was far too much inlined, so I have extracted it.

#[derive(Debug)]
pub struct Assembler {
    pub(crate) pseudo_instruction_table: HashMap<&'static str, &'static PseudoInstruction>,
    pub section_dot_text: Vec<u8>,
    pub section_dot_data: Vec<u8>,
    pub section_dot_rel: Vec<u8>,
    pub section_dot_line: Vec<u8>,
    pub symbol_table: Vec<Symbol>,
    pub(crate) current_section: Section,
    pub(crate) current_address: u32,
    pub(crate) text_address: u32,
    pub(crate) data_address: u32,
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
            current_section: Section::Null,
            current_address: 0,
            text_address: MIPS_TEXT_START_ADDR,
            data_address: MIPS_DATA_START_ADDR,
            most_recent_label: String::from(""),
        }
    }

    /// Add a label to the symbol table with the corresponding value. If a double update was attempted, errors vector will be extended.
    pub(crate) fn add_label(
        &mut self,
        ident: &str,
        value: u32,
        visibility: Visibility,
    ) -> Result<(), ErrorKind> {
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
            value,
            size: 4,
            visibility,
            section: self.current_section.clone(),
        };

        self.symbol_table.push(sym);

        println!("Inserted symbol {} at 0x{:x}", ident, self.current_address);

        self.most_recent_label = ident.to_string();

        Ok(())
    }

    /// Attempt to assemble a parsed line. If successful, add bytes to section .text
    /// else, extend errors and keep it pushing.
    pub fn assemble_instruction(&mut self, instr: &str, args: Vec<AstKind>) -> AssembleResult<()> {
        // look up in our unified map
        let meta = INSTRUCTION_TABLE
            .get(instr)
            .ok_or(ErrorKind::UnknownInstruction(instr.to_string()))?;

        // pull out relocation info from whichever variant we have
        let relocation = meta.relocation_type;

        // if this instruction needs a relocation entry, build it
        if let Some(r_type) = relocation {
            if let Some(AstKind::Symbol(symbol_ident)) =
                args.iter().find(|ast| matches!(ast, AstKind::Symbol(_)))
            {
                let symbol_offset: u32 = self
                    .get_symbol_offset(symbol_ident.to_string())
                    .map_err(|_| ErrorKind::UndefinedSymbol(symbol_ident.clone()))?;

                let new_bytes: Vec<u8> = RelocationEntry {
                    r_offset: self.current_address - MIPS_TEXT_START_ADDR,
                    r_sym: symbol_offset,
                    r_type: r_type.clone(),
                }
                .to_bytes();

                self.section_dot_rel.extend(new_bytes);
            }
        }

        // // convert any Symbol args into a dummy zero immediate for packing
        // let processed_args: Vec<AstKind> = args
        //     .into_iter()
        //     .map(|arg| match arg {
        //         AstKind::Symbol(_) => AstKind::Immediate(0),
        //         other => other,
        //     })
        //     .collect();

        // do the actual packing based on Int vs. Fp
        let packed: RawInstruction = assemble_instruction(meta, /*processed_args*/ args)?;

        // append the bytes to .text
        self.section_dot_text
            .extend_from_slice(&packed.to_be_bytes());
        self.current_address += MIPS_ADDRESS_ALIGNMENT;
        Ok(())
    }

    pub fn assemble_pseudo_instruction(
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
        self.add_data_bytes(&to_push);
    }

    fn add_data_bytes(&mut self, bytes: &[u8]) {
        self.current_address += bytes.len() as u32;
        self.section_dot_data.extend(bytes);

        match self
            .symbol_table
            .iter_mut()
            .find(|s| s.identifier == self.most_recent_label)
        {
            Some(res) => res.size = bytes.len() as u32,
            None => {}
        }
    }

    /// entry point for folding ast into the environment
    pub fn assemble_ast(&mut self, ast: AstKind) -> Result<(), ErrorKind> {
        match ast {
            AstKind::Label(s) => self.add_label(&s, self.current_address, Visibility::Local)?,
            AstKind::Globl(s) => {
                // dbg!(println!("0x{:x}", self.current_address));
                self.add_label(&s, self.current_address, Visibility::Global)?
            }
            AstKind::Asciiz(s) => self.assemble_asciiz(s),
            AstKind::Float(f) => self.add_data_bytes(&f.to_be_bytes(f32::to_be_bytes)),
            AstKind::Double(d) => self.add_data_bytes(&d.to_be_bytes(f64::to_be_bytes)),
            AstKind::Section(section) => match section {
                Section::Text => self.switch_to_text_section(),
                Section::Data => self.switch_to_data_section(),
                _ => panic!("other sections not implemented"),
            },
            AstKind::Instruction(instr, args) => {
                let ast_kinds = args.into_iter().map(|ast| ast.kind).collect();
                if self.pseudo_instruction_table.contains_key(instr.as_str()) {
                    self.assemble_pseudo_instruction(&instr, ast_kinds)?;
                } else {
                    self.assemble_instruction(&instr, ast_kinds)?
                }
            }
            AstKind::Word(word_args) => {
                self.add_data_bytes(&word_args.to_be_bytes(u32::to_be_bytes))
            }
            AstKind::Immediate(_) => panic!(),
            AstKind::Symbol(_) => panic!(),
            AstKind::Register(_) => panic!(),
            AstKind::FpRegister(_) => panic!(),
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
                self.add_label(&ident, 0, Visibility::Local)?;
                return Ok(self.symbol_table.len() as u32);
            }
        };
    }
}
