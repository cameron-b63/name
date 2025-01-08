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

#[derive(Debug, Clone)]
pub struct Span<'a, T> {
    pub src_span: SrcSpan<'a>,
    pub kind: T,
}

impl<'a, T> Span<'a, T> {
    pub fn new(src_span: SrcSpan<'a>, kind: T) -> Self {
        Span { src_span, kind }
    }

    pub fn map<U, F: FnMut(T) -> U>(self, mut f: F) -> Span<'a, U> {
        Span {
            src_span: self.src_span,
            kind: f(self.kind),
        }
    }

    pub fn src_string(&self) -> String {
        self.src_span.src.to_string()
    }
}

impl<'a, T: PartialEq> Span<'a, T> {
    pub fn is_kind(&self, x: T) -> bool {
        self.kind == x
    }
}
