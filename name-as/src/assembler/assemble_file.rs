use name_core::parse::preprocessor::Preprocessor;
use name_core::parse::session::Session;
use std::error::Error;

use crate::assembler::assembler::{AssembleError, Assembler, ErrorKind};
use name_core::{
    constants::{MIPS_ADDRESS_ALIGNMENT, MIPS_DATA_START_ADDR, MIPS_TEXT_START_ADDR},
    elf_def::{RelocationEntry, STT_FUNC, STT_OBJECT},
    instruction::instruction_set::INSTRUCTION_TABLE,
    parse::{
        lexer::{self, Lexer},
        parse::{self, AstKind, Parser},
        span::Span,
        token::{TokenCursor, TokenKind},
    },
    structs::{LineInfo, Section, Symbol, Visibility},
};

use std::path::PathBuf;

pub fn assemble_file<'a>(
    session: &'a mut Session<'a>,
    file_path: PathBuf,
) -> Result<Assembler, ()> {
    let mut should_assemble = true;

    let cont = session.add_file(&file_path);

    let mut lexer = Lexer::new(&cont);
    let (errs, toks) = lexer.lex();

    if !errs.is_empty() {
        for err in errs {
            println!("{}", err);
        }
        return Err(());
    }

    let mut preprocessor = Preprocessor::new(session);
    let ppd = preprocessor.preprocess(toks);

    let mut parser = Parser::new(ppd, cont);

    let (perrs, asts) = parser.parse();

    if !perrs.is_empty() {
        for perr in perrs {
            println!("{}", perr);
        }
        return Err(());
    }

    let mut assembler = Assembler::new();
    let aerrs = assembler.assemble(asts);

    if !aerrs.is_empty() {
        for aerr in aerrs {
            println!("{}", aerr);
        }
        return Err(());
    }

    // process line info
    for line in cont.split('\n') {
        let start_address = match assembler.current_section {
            Section::Text => assembler.current_address,
            Section::Data => assembler.text_address,
            Section::Null => 0,
        };

        // Extend section .line to include the new line
        assembler.section_dot_line.extend(
            LineInfo {
                content: line.to_string(),
                line_number: assembler.line_number as u32,
                start_address: match assembler.current_section {
                    Section::Text => start_address,
                    _ => 0,
                },
                end_address: match assembler.current_section {
                    Section::Text => assembler.current_address,
                    Section::Data => assembler.text_address,
                    _ => 0,
                },
            }
            .to_bytes(),
        );

        assembler.line_number += 1;
    }

    Ok(assembler)
}
