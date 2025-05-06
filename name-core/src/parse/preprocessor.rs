use crate::parse::lexer::Lexer;
use crate::parse::session::Session;
use crate::parse::token::{Token, TokenCursor, TokenKind};
use std::collections::{HashMap, VecDeque};

pub struct Preprocessor<'sess, 'sess_ref> {
    sess: &'sess_ref mut Session<'sess>,
    eqvs: HashMap<&'sess str, Vec<Token>>,
}

impl<'sess, 'sess_ref> Preprocessor<'sess, 'sess_ref> {
    pub fn new(sess: &'sess_ref mut Session<'sess>) -> Self {
        Preprocessor {
            sess,
            eqvs: HashMap::new(),
        }
    }

    /// The preprocessor is responsible for expanding our expandables.
    /// Anything that performs something akin to string replacement happens here.
    pub fn preprocess(&mut self, mut cursor: TokenCursor) -> TokenCursor {
        // let mut cursor = TokenCursor::new(toks);
        let mut tokens = VecDeque::new();

        while let Some(tok) = cursor.next() {
            match tok.kind {
                TokenKind::Directive => match self.sess.get_src_str(&tok.src_span) {
                    ".include" => {
                        let src_span = &cursor.next_if(TokenKind::String).unwrap().src_span;
                        let src = self.sess.get_src_str(src_span);
                        let file_name = &src[1..src.len() - 1];

                        let file = match self.sess.add_file(self.sess.dir.join(file_name)) {
                            Ok(f) => f,
                            Err(e) => panic!("{e}"), // TODO: More graceful handling the way it's supposed to be. Pushed back for now because I know preprocessor is still in works.
                        };

                        let mut lexer = Lexer::new(&file.str, file.src_span.pos);
                        //Todo: handle errors
                        let (_errs, cursor) = lexer.lex();

                        let pre = self.preprocess(cursor);
                        tokens.extend(pre.toks);
                    }
                    ".eqv" => {
                        let src_span = &cursor.next_if(TokenKind::Ident).unwrap().src_span;
                        let ident = self.sess.get_src_str(src_span);

                        let mut expansion = Vec::new();

                        // read until and consume the newline but ignore it
                        while let Some(tok) =
                            cursor.next().filter(|tok| !tok.is_kind(TokenKind::Newline))
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
                    _ => tokens.push_back(tok.clone()),
                },
                TokenKind::Ident => {
                    let src = self.sess.get_src_str(&tok.src_span);
                    if let Some(expansion) = self.eqvs.get(src) {
                        tokens.extend(expansion.clone());
                    } else {
                        tokens.push_back(tok.clone())
                    }
                }
                _ => tokens.push_back(tok.clone()),
            }
        }
        TokenCursor::new(tokens)
    }
}
