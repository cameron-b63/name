use name_core::parse::preprocessor::Preprocessor;
use name_core::parse::session::Session;

use crate::assembler::assembler::Assembler;
use name_core::{
    parse::{lexer::Lexer, parse::Parser},
    structs::{LineInfo, Section},
};

use std::path::PathBuf;

/// Assembles a given file using a parser session.
/// Takes a parser session containing the bump allocator and the parent directory.
/// Outputs an Assembler state from which the ELF object file will be derived.
pub fn assemble_file<'a>(
    session: &'a mut Session<'a>,
    file_path: PathBuf,
) -> Result<Assembler, ()> {
    // Add the given infile to the parser session.
    let cont = session.add_file(&file_path);

    // Create a lexer on the file content to tokenize it.
    let mut lexer = Lexer::new(&cont);
    let (errs, toks) = lexer.lex();

    // If lexer errors occur, exit early with all of them.
    // These are worse than AST-level errors, and going further in the process generates
    // less useful output than simply exiting early.
    if !errs.is_empty() {
        for err in errs {
            println!("{}", err);
        }
        return Err(());
    }

    // Run the preprocessor over the parser session.
    let mut preprocessor = Preprocessor::new(session);
    // Preprocess the lexed tokens to handle .include, .eqv, .macro, etc.
    let ppd = preprocessor.preprocess(toks);

    // Create a new parser using the preprocessed (expanded) tokens and the file content.
    let mut parser = Parser::new(ppd, cont);

    // Run the parsing operation, generating the AST.
    let (perrs, asts) = parser.parse();

    // If there were errors in generating the AST, print them and return.
    // This constitutes an early exit because after this point, generated errors
    // would likely be redundant or otherwise malformed. "We have bigger fish to fry."
    if !perrs.is_empty() {
        for perr in perrs {
            println!("{}", perr);
        }
        return Err(());
    }

    // Create an Assembler state to fold the AST into.
    let mut assembler = Assembler::new();
    // This is the point at which "assembly" actually occurs.
    let aerrs = assembler.assemble(asts);

    // If there exist errors in the folding process, print them and return.
    // This is the last point of user error.
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

    // Return the assembler state.
    Ok(assembler)
}
