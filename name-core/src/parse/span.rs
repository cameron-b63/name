use std::fmt;
use std::ops::Range;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SrcPos {
    pub pos: usize,
    pub line_pos: usize,
    pub line: usize,
}

impl fmt::Display for SrcPos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.line_pos)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct SrcSpan {
    pub start: SrcPos,
    pub end: SrcPos,
}

impl SrcSpan {
    pub fn range(&self) -> Range<usize> {
        self.start.pos..self.end.pos
    }

    pub fn combine(&self, other: &Self) -> Self {
        SrcSpan {
            start: self.start.clone(),
            end: other.end.clone(),
        }
    }
}

impl fmt::Display for SrcSpan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.start)
    }
}

#[derive(Debug, Clone)]
pub struct Span<T> {
    pub src_span: SrcSpan,
    pub kind: T,
}

impl<T> Span<T> {
    pub fn new(src_span: SrcSpan, kind: T) -> Self {
        Span { src_span, kind }
    }

    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Span<U> {
        Span {
            src_span: self.src_span,
            kind: f(self.kind),
        }
    }
}

impl<T: PartialEq> Span<T> {
    pub fn is_kind(&self, x: T) -> bool {
        self.kind == x
    }
}
