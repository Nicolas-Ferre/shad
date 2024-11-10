use derivative::Derivative;
use std::rc::Rc;

/// The span of a group of token in a file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    /// The byte offset of the span start.
    pub start: usize,
    /// The byte offset of the span end.
    pub end: usize,
    /// The module in which the tokens are located.
    pub module: Rc<ModuleLocation>,
}

impl Span {
    /// Creates a span.
    pub fn new(start: usize, end: usize, module: Rc<ModuleLocation>) -> Self {
        Self { start, end, module }
    }

    /// Join two spans.
    pub fn join(start: &Self, end: &Self) -> Self {
        Self {
            start: start.start,
            end: end.end,
            module: start.module.clone(),
        }
    }
}

/// The location of a Shad module.
#[derive(Debug, Clone, Derivative)]
#[derivative(PartialEq, Eq, Hash)]
pub struct ModuleLocation {
    /// The name of the module.
    pub name: String,
    /// The file path of the module.
    #[derivative(PartialEq = "ignore")]
    #[derivative(Hash = "ignore")]
    pub path: String,
    /// The code of the module.
    #[derivative(PartialEq = "ignore")]
    #[derivative(Hash = "ignore")]
    pub code: String,
}
