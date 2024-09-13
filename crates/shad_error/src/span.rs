/// The span of a group of token in a file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    /// The byte offset of the span start.
    pub start: usize,
    /// The byte offset of the span end.
    pub end: usize,
}

impl Span {
    /// Creates a span.
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}
