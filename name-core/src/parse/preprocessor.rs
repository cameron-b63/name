use crate::constants::MIPS_TEXT_START_ADDR;
use crate::elf_utils::create_serialized_line_information;
use crate::instruction::helpers::is_standard_instruction;
use crate::instruction::pseudo_instruction_set::PSEUDO_INSTRUCTION_SET;
use crate::parse::lexer::Lexer;
use crate::parse::session::Session;
use crate::parse::span::Span;
use crate::parse::token::{Token, TokenCursor, TokenKind};
use crate::structs::LineInfo;
use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::process::exit;

/// The preprocessor struct is pretty much an extension of a parser session.
/// It contains a few additional fields solely related to preprocessing,
/// but must always be constructed with some parser session.
///
/// The serialized line information is public so that the caller can access it
/// in order to construct section .line in the output executable.
pub struct Preprocessor<'sess, 'sess_ref> {
    sess: &'sess_ref mut Session<'sess>,
    eqvs: HashMap<&'sess str, Vec<Token>>,
    _expandable_sizes: HashMap<&'sess str, usize>,
    pub line_information: Vec<LineInfo>,
    pub filenames: Vec<PathBuf>,
    pub current_file_index: usize,
    pub current_file_name: Option<PathBuf>,
}

impl<'sess, 'sess_ref> Preprocessor<'sess, 'sess_ref> {
    pub fn new(sess: &'sess_ref mut Session<'sess>) -> Self {
        Preprocessor {
            sess,
            eqvs: HashMap::new(),
            _expandable_sizes: HashMap::new(),
            line_information: Vec::new(),
            filenames: Vec::new(),
            current_file_index: 0,
            current_file_name: None,
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
    pub fn preprocess(&mut self, cursor: TokenCursor) -> TokenCursor {
        // Detect expandables
        let _ = self.expandable_detection_pass(&cursor);

        // Gather LineInfo
        let _ = self.lineinfo_creation_pass(&cursor);

        // Carry out expansions
        let tokens = self.token_expansion_pass(&cursor);

        // Return the new TokenCursor with tokens of expanded content
        return TokenCursor::new(tokens);
    }

    /// First pass: Find expandable tokens.
    /// Move through the file token-by-token, matching on directives that declare expandables.
    /// If you're familiar with the initial iteration of logic, this is just a decoupling of
    /// the "search" pass and the "expand" pass.
    fn expandable_detection_pass(&mut self, parent_cursor: &TokenCursor) {
        // Create temporary cursor for this pass
        let mut expandable_pass_cursor = parent_cursor.clone();
        // Move through the file token-by-token
        while let Some(tok) = expandable_pass_cursor.next() {
            match tok.kind {
                TokenKind::Directive => match self.sess.get_src_str(&tok.src_span) {
                    // For now, we will assume that included files DO NOT contain program text.
                    // This will need to be fixed later.
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
    }

    /// Second pass:
    /// simply create line information by iterating through tokens and delimiting by TokenKind::Newline.
    fn lineinfo_creation_pass(&mut self, parent_cursor: &TokenCursor) {
        // needed temp variables
        let mut lineinfo_cursor: TokenCursor = parent_cursor.clone(); // Take advantage of the cursor type
        let mut line_number: u32 = 1; // Line number for serializing line information
        let mut dummy_pc: u32 = MIPS_TEXT_START_ADDR; // PC for serializing line information (line<->PC relationship)
        let mut last_text: u32 = 0; // Used for proper switching back and forth between .text and .data
        let mut increment_pc_by: u32 = 0; // Tracker for each line to see how much to add to dummy PC (0 if no instruction)

        // Initialize the line information with the parent file we were given
        let file_name = match &self.current_file_name {
            Some(f) => f,
            None => &self.sess.get_parent_file(),
        };

        self.filenames.push(file_name.clone());

        // Gather the line information
        while let Some(tok) = lineinfo_cursor.next() {
            match tok.kind {
                // If the token is an ident, it might be an instruction. Let's find out.
                TokenKind::Ident => {
                    // First, make the common case fast and see if token is a normal instruction.
                    if is_standard_instruction(self.sess.get_src_str(&tok.src_span)) {
                        increment_pc_by = 4;
                    }
                    // If it wasn't a standard instruction, it's probably a pesudo instruction.
                    else if let Some(pseudo_instruction_information) = PSEUDO_INSTRUCTION_SET
                        .iter()
                        .find(|info| info.mnemonic == self.sess.get_src_str(&tok.src_span))
                    {
                        increment_pc_by =
                            4 * pseudo_instruction_information.lines_expanded_to as u32;
                    }
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
                    self.line_information.push(LineInfo {
                        file_table_index: self.current_file_index as u32,
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
    }

    /// Third pass:
    /// Consume declaration of expansions and expand uses of expandables (lots of expanding going on...)
    /// Token-level expansion is performed instead of string replacement.
    fn token_expansion_pass(&mut self, parent_cursor: &TokenCursor) -> VecDeque<Span<TokenKind>> {
        // Declare a cursor for use in this pass
        let mut cursor: TokenCursor = parent_cursor.clone();

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

                        let file = match self.sess.add_file(self.sess.dir.join(file_name)) {
                            Ok(f) => f,
                            // If an error occurred in opening file, fix it.
                            Err(e) => {
                                self.sess.report_error(&e.to_string(), src_span);
                                exit(1);
                            }
                        };

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
                        self.current_file_index += 1;
                        self.current_file_name = Some(PathBuf::from(file_name));
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

        tokens
    }

    pub fn get_serialized_line_information(&mut self) -> Vec<u8> {
        create_serialized_line_information(&self.line_information, &self.filenames)
    }
}
