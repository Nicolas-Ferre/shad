use crate::type_::Type;
use crate::{AnalyzedTypes, ErrorLevel, LocatedMessage, SemanticError};
use fxhash::FxHashMap;
use shad_parser::{BufferItem, Item, ParsedProgram, Span};
use std::rc::Rc;

/// All buffers found when analysing a Shad program.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AnalyzedBuffers {
    /// The buffers.
    pub buffers: FxHashMap<String, Rc<Buffer>>,
    /// The semantic errors related to buffers.
    pub errors: Vec<SemanticError>,
}

impl AnalyzedBuffers {
    pub(crate) fn new(parsed: &ParsedProgram, types: &AnalyzedTypes) -> Self {
        let mut buffers = FxHashMap::default();
        let mut errors = vec![];
        for item in &parsed.items {
            let Item::Buffer(buffer) = item;
            let value_type = types.expr_type(&buffer.value);
            let existing_buffer = buffers.insert(
                buffer.name.label.clone(),
                Rc::new(Buffer::new(buffer, buffers.len(), value_type)),
            );
            if let Some(existing_buffer) = existing_buffer {
                errors.push(duplicated_name_error(buffer, &existing_buffer, parsed));
            }
        }
        Self { buffers, errors }
    }
}

/// An analyzed buffer.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Buffer {
    /// The final name that will be used in shaders.
    pub final_name: String,
    /// The buffer type.
    pub type_: Rc<Type>,
    /// The buffer initial value.
    pub value: shad_parser::Expr,
    /// The span of the buffer name in the initial Shad code.
    pub name_span: Span,
}

impl Buffer {
    pub(crate) fn new(buffer: &BufferItem, index: usize, value_type: &Rc<Type>) -> Self {
        Self {
            final_name: format!("buf{index}"),
            type_: value_type.clone(),
            value: buffer.value.clone(),
            name_span: buffer.name.span,
        }
    }
}

pub(crate) fn duplicated_name_error(
    buffer: &BufferItem,
    existing_buffer: &Buffer,
    parsed: &ParsedProgram,
) -> SemanticError {
    SemanticError::new(
        format!(
            "buffer with name `{}` is defined multiple times",
            buffer.name.label
        ),
        vec![
            LocatedMessage::new(
                ErrorLevel::Error,
                buffer.name.span,
                "duplicated buffer name",
            ),
            LocatedMessage::new(
                ErrorLevel::Info,
                existing_buffer.name_span,
                "buffer with same name is defined here",
            ),
        ],
        parsed,
    )
}
