use crate::parse::span::SrcSpan;
use bumpalo::Bump;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct File {
    path: PathBuf,
    pub str: String,
    pub src_span: SrcSpan,
    lines: Vec<SrcSpan>,
}

impl File {
    pub fn new(path: PathBuf, str: String, pos: usize) -> Self {
        let mut last_pos = pos;

        let lines: Vec<SrcSpan> = str
            .lines()
            .map(|line| {
                let pos = last_pos;
                let length = line.len() + 1;
                last_pos += length;
                SrcSpan { pos, length }
            })
            .collect();

        File {
            path,
            src_span: SrcSpan {
                pos,
                length: str.len(),
            },
            str,
            lines,
        }
    }

    pub fn get_line_column_str(&self, src_span: &SrcSpan) -> (usize, usize, &str) {
        let line_idx = self
            .lines
            .binary_search_by(|cmp_src_span| cmp_src_span.pos.cmp(&src_span.pos))
            .unwrap_or_else(|line_idx| line_idx - 1);

        let column = src_span.pos - self.lines[line_idx].pos;

        let start = src_span.pos - self.src_span.pos;

        let str = &self.str.get(start..(start + src_span.length)).unwrap_or("");

        (line_idx + 1, column, str)
    }
}

#[derive(Debug)]
pub struct Src<'a> {
    files: Vec<&'a File>,
    pub length: usize,
}

impl<'a> Src<'a> {
    pub fn new() -> Self {
        Src {
            files: Vec::new(),
            length: 0,
        }
    }

    pub fn add_file(&mut self, file: &'a File) {
        self.length = self.length + file.src_span.length;
        self.files.push(file);
    }

    pub fn get_span_details(&self, src_span: &SrcSpan) -> (&'a Path, &'a str, usize, usize) {
        // dbg!(&self);
        let file_idx = self
            .files
            .binary_search_by(|file| file.src_span.pos.cmp(&src_span.pos))
            .unwrap_or_else(|file_idx| file_idx - 1);

        let file = self.files[file_idx];

        let (line, column, str) = file.get_line_column_str(src_span);

        (&file.path, str, line, column)
    }

    pub fn get_str(&self, src_span: &SrcSpan) -> &'a str {
        let (_, cont, _, _) = self.get_span_details(src_span);
        cont
    }
}

#[derive(Debug)]
pub struct Session<'a> {
    bump: &'a Bump,
    pub src: Src<'a>,
    pub dir: PathBuf,
    pub should_assemble: bool,
}

impl<'a> Session<'a> {
    pub fn new(bump: &'a Bump, dir: PathBuf) -> Session<'a> {
        Session {
            bump,
            src: Src::new(),
            dir,
            should_assemble: true,
        }
    }

    pub fn add_file(&mut self, path: PathBuf) -> &'a File {
        let cont = fs::read_to_string(&path).unwrap();
        let file = self.bump.alloc(File::new(path, cont, self.src.length));
        self.src.add_file(file);
        file
    }

    pub fn get_src_str(&self, src_span: &SrcSpan) -> &'a str {
        self.src.get_str(src_span)
    }

    pub fn report_error(&mut self, err: &str, src_span: &SrcSpan) {
        self.should_assemble = false;

        let (file, str, mut line, col) = self.src.get_span_details(&src_span);

        println!("error: {}\n\t--> {:?}:{}:{}", err, file, line, col);

        for str_line in str.lines() {
            println!("\t{} | {}", line, str_line);
            line += 1;
        }
    }
}
