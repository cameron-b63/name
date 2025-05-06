use std::ops::Range;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SrcSpan {
    pub pos: usize,
    pub length: usize,
}

impl SrcSpan {
    pub fn range(&self) -> Range<usize> {
        self.pos..(self.pos + self.length)
    }

    pub fn combine(&self, other: &Self) -> Self {
        SrcSpan {
            pos: self.pos,
            length: other.pos - self.pos + other.length,
        }
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
