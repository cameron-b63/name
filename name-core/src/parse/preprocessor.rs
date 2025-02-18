use crate::parse::lexer::Lexer;
use crate::parse::session::Session;
use crate::parse::token::{Token, TokenCursor, TokenKind};
use std::collections::HashMap;

pub struct Preprocessor<'a> {
    sess: &'a mut Session<'a>,
    eqvs: HashMap<&'a str, Vec<Token<'a>>>,
}

impl<'a> Preprocessor<'a> {
    pub fn new(sess: &'a mut Session<'a>) -> Self {
        Preprocessor {
            sess,
            eqvs: HashMap::new(),
        }
    }

    pub fn preprocess(&mut self, toks: Vec<Token<'a>>) -> Vec<Token<'a>> {
        let mut cursor = TokenCursor::new(toks);
        let mut tokens = Vec::new();

        while let Some(tok) = cursor.next() {
            match tok.token.kind {
                TokenKind::Directive => match tok.src {
                    ".include" => {
                        let src = cursor.next_if(TokenKind::String).unwrap().src;
                        let file_name = &src[1..src.len() - 1];
                        let cont = self.sess.add_file(&self.sess.dir.join(file_name));

                        let mut lexer = Lexer::new(cont);
                        let (errs, toks) = lexer.lex();

                        let pre = self.preprocess(toks);
                        tokens.extend(pre);
                    }
                    ".eqv" => {
                        let ident = cursor.next_if(TokenKind::Ident).unwrap().src;
                        let mut expansion = Vec::new();

                        // read until and consume the newline but ignore it
                        while let Some(tok) = cursor
                            .next()
                            .filter(|tok| !tok.token.is_kind(TokenKind::Newline))
                        {
                            if let Some(c_expansion) = self.eqvs.get(tok.src) {
                                expansion.extend(expansion.clone());
                            } else {
                                expansion.push(tok.clone())
                            }
                        }

                        let _ = self.eqvs.insert(ident, expansion);
                    }
                    ".macro" => todo!("implement macros"),
                    _ => tokens.push(tok.clone()),
                },
                TokenKind::Ident => {
                    if let Some(expansion) = self.eqvs.get(tok.src) {
                        tokens.extend(expansion.clone());
                    } else {
                        tokens.push(tok.clone())
                    }
                }
                _ => tokens.push(tok.clone()),
            }
        }
        tokens
    }
}
