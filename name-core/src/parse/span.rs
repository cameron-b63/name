use std::fmt;

#[derive(Debug, PartialEq, Clone)]
pub struct SrcSpan<'a> {
    pub start: usize,
    pub end: usize,
    pub src: &'a str,
    pub line: usize,
    pub line_pos: usize,
}

impl fmt::Display for SrcSpan<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{} {}", self.line, self.line_pos, self.src)
    }
}

pub struct Span<'a, T> {
    src_span: SrcSpan<'a>,
    unspan: T,
}

impl<'a, T> Span<'a, T> {
    pub fn new(src_span: SrcSpan<'a>, unspan: T) -> Self {
        Span { src_span, unspan }
    }

    pub fn map<U, F: FnMut(T) -> U>(self, mut f: F) -> Span<'a, U> {
        Span {
            src_span: self.src_span,
            unspan: f(self.unspan),
        }
    }
}
