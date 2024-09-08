use crate::type_::Type;
use crate::{AnalyzedTypes, ErrorLevel, LocatedMessage, SemanticError};
use fxhash::FxHashMap;
use shad_parser::{BufferItem, Ident, Item, ParsedProgram};
use std::rc::Rc;

/// All buffers found when analysing a Shad program.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AnalyzedBuffers {
    /// The buffers.
    pub buffers: Vec<Rc<Buffer>>,
    /// The mapping between Shad buffer names and buffer index.
    pub buffer_name_indexes: FxHashMap<String, usize>,
    /// The semantic errors related to buffers.
    pub errors: Vec<SemanticError>,
}

impl AnalyzedBuffers {
    pub(crate) fn new(parsed: &ParsedProgram, types: &AnalyzedTypes) -> Self {
        let mut buffers = vec![];
        let mut buffer_name_indexes = FxHashMap::default();
        let mut errors = vec![];
        for item in &parsed.items {
            let Item::Buffer(buffer) = item;
            let buffer_index = buffers.len();
            let value_type = types.expr_type(&buffer.value);
            let existing_index =
                buffer_name_indexes.insert(buffer.name.label.clone(), buffer_index);
            buffers.push(Rc::new(Buffer::new(buffer, buffer_index, value_type)));
            if let Some(index) = existing_index {
                errors.push(duplicated_name_error(buffer, &buffers[index], parsed));
            }
        }
        Self {
            buffers,
            buffer_name_indexes,
            errors,
        }
    }
}

/// An analyzed buffer.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Buffer {
    /// The unique buffer index.
    pub index: usize,
    /// The buffer type.
    pub type_: Rc<Type>,
    /// The buffer initial value.
    pub value: shad_parser::Expr,
    /// The buffer name in the initial Shad code.
    pub name: Ident,
}

impl Buffer {
    pub(crate) fn new(buffer: &BufferItem, index: usize, value_type: &Rc<Type>) -> Self {
        Self {
            index,
            type_: value_type.clone(),
            value: buffer.value.clone(),
            name: buffer.name.clone(),
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
                existing_buffer.name.span,
                "buffer with same name is defined here",
            ),
        ],
        parsed,
    )
}
