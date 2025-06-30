use name_core::parse::preprocessor::Preprocessor;
use name_core::parse::session::Session;

use crate::assembler::assembler::Assembler;
use name_core::parse::{lexer::Lexer, parse::Parser};

use std::path::PathBuf;

/// Assembles a given file using a parser session.
/// Takes a parser session containing the bump allocator and the parent directory.
/// Outputs an Assembler state from which the ELF object file will be derived.
pub fn assemble_file<'sess, 'sess_ref>(
    session: &'sess_ref mut Session<'sess>,
    file_path: PathBuf,
) -> Result<Assembler, ()> {
    // Add the given infile to the parser session.
    let file = session.add_file(file_path);

    // Create a lexer on the file content to tokenize it.
    let mut lexer = Lexer::new(&file.str, 0);
    let (errs, toks) = lexer.lex();

    // If lexer errors occur, exit early with all of them.
    // These are worse than AST-level errors, and going further in the process generates
    // less useful output than simply exiting early.
    if !errs.is_empty() {
        for err in errs {
            session.report_error(&format!("{}", err.kind), &err.src_span);
        }
        return Err(());
    }

    // Run the preprocessor over the (tokenized) parser session.
    let ppd = Preprocessor::new(session).preprocess(toks);

    // Preprocess the lexed tokens to handle .include, .eqv, .macro, etc.

    // Create a new parser using the preprocessed (expanded) tokens and the file content.
    let mut parser = Parser::new(ppd, session);

    // Run the parsing operation, generating the AST.
    let (perrs, asts) = parser.parse();

    // If there were errors in generating the AST, print them and return.
    // This constitutes an early exit because after this point, generated errors
    // would likely be redundant or otherwise malformed. "We have bigger fish to fry."
    if !perrs.is_empty() {
        for perr in perrs {
            session.report_error(&format!("{}", perr.kind), &perr.src_span);
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
            session.report_error(&format!("{}", aerr.kind), &aerr.src_span);
        }
        return Err(());
    }

    // Return the assembler state.
    Ok(assembler)
}
