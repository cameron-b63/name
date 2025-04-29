use crate::parse::span::Span;
use bumpalo::Bump;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct Session<'a> {
    bump: &'a Bump,
    buffers: Vec<&'a str>,
    pub dir: PathBuf,
    pub should_assemble: bool,
}

impl<'a> Session<'a> {
    pub fn new(bump: &'a Bump, dir: PathBuf) -> Session<'a> {
        Session {
            bump,
            buffers: Vec::new(),
            dir,
            should_assemble: true,
        }
    }

    pub fn add_file(&mut self, path: &Path) -> &'a str {
        let cont = fs::read_to_string(&path).unwrap();
        self.add_src(cont)
    }

    pub fn add_src(&mut self, cont: String) -> &'a str {
        let src = self.bump.alloc(cont);
        self.buffers.push(src);
        src
    }

    pub fn report_error<T: fmt::Display>(&mut self, error: Span<T>) {
        self.should_assemble = false;
        println!("{}", error);
    }
}
