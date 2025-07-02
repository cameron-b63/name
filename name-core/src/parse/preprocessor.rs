use crate::constants::MIPS_TEXT_START_ADDR;
use crate::elf_utils::create_serialized_line_information;
use crate::instruction::helpers::is_standard_instruction;
use crate::parse::lexer::Lexer;
use crate::parse::session::Session;
use crate::parse::span::SrcSpan;
use crate::parse::token::{Token, TokenCursor, TokenKind};
use crate::structs::LineInfo;
use std::collections::{HashMap, VecDeque};
use std::process::exit;

pub struct Preprocessor<'sess, 'sess_ref> {
    sess: &'sess_ref mut Session<'sess>,
    eqvs: HashMap<&'sess str, Vec<Token>>,
    _expandable_sizes: HashMap<&'sess str, usize>,
    pub serialized_line_information: Vec<u8>,
}

impl<'sess, 'sess_ref> Preprocessor<'sess, 'sess_ref> {
    pub fn new(sess: &'sess_ref mut Session<'sess>) -> Self {
        Preprocessor {
            sess,
            eqvs: HashMap::new(),
            _expandable_sizes: HashMap::new(),
            serialized_line_information: Vec::new(),
        }
    }

    /// The preprocessor is responsible for:
    ///  - creating line information
    ///  - expanding our expandables
    ///
    /// Anything that performs something akin to string replacement happens here (just on the token level).
    ///
    /// To accomplish these goals, the preprocessor performs multiple passes:
    ///  - Pass 1: Identify expandable symbols
    ///  - Pass 2: Generate line information for section .line in output
    ///  - Pass 3: Carry out expansions
    pub fn preprocess(&mut self, mut cursor: TokenCursor) -> TokenCursor {
        // First pass: Find expandable tokens.
        // Move through the file token-by-token, matching on directives that declare expandables.
        // If you're familiar with the initial iteration of logic, this is just a decoupling of
        // the "search" pass and the "expand" pass.
        let mut expandable_pass_cursor: TokenCursor = cursor.clone();

        // Move through the file token-by-token
        while let Some(tok) = expandable_pass_cursor.next() {
            match tok.kind {
                TokenKind::Directive => match self.sess.get_src_str(&tok.src_span) {
                    // For now, we will assume that included files DO NOT contain program text.
                    // This will probably need to be fixed later.
                    ".include" => {}
                    // If an eqv was declared, add it to the eqvs list
                    ".eqv" => {
                        let src_span = &expandable_pass_cursor
                            .next_if(TokenKind::Ident)
                            .unwrap()
                            .src_span;
                        let ident = self.sess.get_src_str(src_span);

                        let mut expansion = Vec::new();

                        // read until and consume the newline but ignore it
                        while let Some(tok) = expandable_pass_cursor
                            .next()
                            .filter(|tok| !tok.is_kind(TokenKind::Newline))
                        {
                            let src = self.sess.get_src_str(&tok.src_span);
                            if let Some(c_expansion) = self.eqvs.get(src) {
                                expansion.extend(c_expansion.clone());
                            } else {
                                expansion.push(tok.clone())
                            }
                        }

                        let _ = self.eqvs.insert(ident, expansion);
                    }
                    ".macro" => todo!("implement macros"),
                    _ => {
                        // Nothing in this pass.
                    }
                },
                _ => {
                    // Nothing in this pass.
                }
            }
        }

        // Second pass: simply create line information by iterating through tokens and delimiting TokenKind::Newline
        let mut lineinfo_cursor: TokenCursor = cursor.clone(); // Take advantage of the cursor type
        let mut line_number = 1; // Line number for serializing line information
        let mut dummy_pc = MIPS_TEXT_START_ADDR; // PC for serializing line information (line<->PC relationship)
        let mut last_text = 0; // Used for proper switching back and forth between .text and .data
        let mut increment_pc_by = 0; // Tracker for each line to see how much to add to dummy PC (0 if no instruction)

        // Initialize the line information with the current file we were given
        let file_name = match cursor.peek() {
            Some(found) => self.sess.src.get_span_details(&found.src_span).0,
            None => {
                self.sess.report_error(
                    "[*] Line info builder found nothing to work with.",
                    &SrcSpan::default(),
                );
                exit(0);
            }
        };

        let mut line_information: Vec<LineInfo> = Vec::new();

        // Gather the line information
        while let Some(tok) = lineinfo_cursor.next() {
            match tok.kind {
                // If the token is an ident, it might be an instruction. Let's find out.
                TokenKind::Ident => {
                    // First, make the common case fast and see if token is a normal instruction.
                    if is_standard_instruction(self.sess.get_src_str(&tok.src_span)) {
                        increment_pc_by = 4;
                    }

                    // TODO: Handle pseudoinstructions, etc.
                }
                TokenKind::Directive => {
                    // LineInfo should ideally not be creating pc mapping unless it really matters.
                    match self.sess.get_src_str(&tok.src_span) {
                        ".data" => {
                            last_text = dummy_pc;
                            dummy_pc = 0;
                        }
                        ".text" => {
                            if last_text == 0 {
                                dummy_pc = MIPS_TEXT_START_ADDR;
                            } else {
                                dummy_pc = last_text;
                            }
                        }
                        _ => {}
                    }
                }
                // If the token is a newline, we must update the line number and maybe the dummy program counter too.
                TokenKind::Newline => {
                    // Add line information to vector
                    line_information.push(LineInfo {
                        file_table_index: 1,
                        start_address: dummy_pc,
                        end_address: dummy_pc + increment_pc_by,
                        line_number,
                    });

                    // If dummy pc needs to be updated, update it
                    dummy_pc += increment_pc_by;
                    increment_pc_by = 0;

                    // Increment line number and get ready for next run
                    line_number += 1;
                }
                // Anything else doesn't really matter to us for right now.
                // Our main concern is creating a mapping from program_counter to line_number.
                _ => {
                    // Nothing yet...
                }
            }
        }

        // Create serialized line information from collected information
        self.serialized_line_information =
            create_serialized_line_information(line_information, file_name.to_path_buf());

        // Pass 3: Carry out expansions
        // Declare a deque to handle expansions
        let mut tokens = VecDeque::new();

        // Perform expansion of all wonderful things
        while let Some(tok) = cursor.next() {
            match tok.kind {
                TokenKind::Directive => match self.sess.get_src_str(&tok.src_span) {
                    // Consume all the ".eqv" content, as it's already been handled and doesn't need to be pushed through.
                    ".eqv" => {
                        // Consume everything
                        while let Some(_) =
                            cursor.next().filter(|tok| !tok.is_kind(TokenKind::Newline))
                        {
                            // Nothing to do except consume.
                        }
                    }
                    // Handle include by creating a mini-lexer and pushing lexed content to current session.
                    ".include" => {
                        let src_span = &cursor.next_if(TokenKind::String).unwrap().src_span;
                        let src = self.sess.get_src_str(src_span);
                        let file_name = &src[1..src.len() - 1];

                        let file = self.sess.add_file(self.sess.dir.join(file_name));

                        // Create mini-lexer to handle included file
                        let mut lexer = Lexer::new(&file.str, file.src_span.pos);
                        let (errs, cursor) = lexer.lex();

                        // If errors occurred, print them (but do not exit)
                        if !errs.is_empty() {
                            println!("[*] Errors found in included file during preprocessing: ");
                            println!("[*] In file {:?}:", file_name);
                            errs.iter().for_each(|err| {
                                self.sess
                                    .report_error(format!("{err:?}").as_str(), src_span)
                            })
                        }

                        // Start a new preprocessor on the included file
                        // WARNING: Recursion possible! Please account for this if plausible.
                        let pre = self.preprocess(cursor);

                        // This is the part where the lexed content from the included file gets handled.
                        tokens.extend(pre.toks);
                    }
                    _ => tokens.push_back(tok.clone()),
                },
                // Handle ".eqv" uses with eqvs Vec
                TokenKind::Ident => {
                    let src = self.sess.get_src_str(&tok.src_span);
                    // If there's an expansion associated with this symbol, allow the expansion to happen.
                    if let Some(expansion) = self.eqvs.get(src) {
                        tokens.extend(expansion.clone());
                    } else {
                        tokens.push_back(tok.clone())
                    }
                }
                _ => tokens.push_back(tok.clone()),
            }
        }

        // Return the new TokenCursor with tokens of expanded content
        TokenCursor::new(tokens)
    }
}
