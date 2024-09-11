use crate::type_::Type;
use crate::{AnalyzedTypes, ErrorLevel, LocatedMessage, SemanticError};
use fxhash::FxHashMap;
use shad_parser::{Ast, AstBufferItem, AstIdent, AstItem};
use std::rc::Rc;

/// All buffers found when analysing a Shad program.
#[derive(Default, Debug, PartialEq, Eq, Clone)]
pub struct AnalyzedBuffers {
    /// The buffers.
    pub buffers: Vec<Rc<Buffer>>,
    /// The mapping between Shad buffer names and buffer index.
    pub buffer_name_indexes: FxHashMap<String, usize>,
    /// The semantic errors related to buffers.
    pub errors: Vec<SemanticError>,
}

impl AnalyzedBuffers {
    pub(crate) fn init(&mut self, ast: &Ast, types: &AnalyzedTypes) {
        for item in &ast.items {
            let AstItem::Buffer(buffer) = item;
            let buffer_index = self.buffers.len();
            let value_type = types.expr_type(&buffer.value, self);
            let existing_index = self
                .buffer_name_indexes
                .insert(buffer.name.label.clone(), buffer_index);
            self.buffers
                .push(Rc::new(Buffer::new(buffer, buffer_index, value_type)));
            if let Some(index) = existing_index {
                self.errors
                    .push(self.duplicated_name_error(buffer, index, ast));
            }
        }
    }

    pub(crate) fn find(&self, name: &str) -> Option<&Rc<Buffer>> {
        self.buffer_name_indexes
            .get(name)
            .map(|&index| &self.buffers[index])
    }

    fn duplicated_name_error(
        &self,
        buffer: &AstBufferItem,
        existing_index: usize,
        ast: &Ast,
    ) -> SemanticError {
        SemanticError::new(
            format!(
                "buffer with name `{}` is defined multiple times",
                buffer.name.label
            ),
            vec![
                LocatedMessage {
                    level: ErrorLevel::Error,
                    span: buffer.name.span,
                    text: "duplicated buffer name".into(),
                },
                LocatedMessage {
                    level: ErrorLevel::Info,
                    span: self.buffers[existing_index].name.span,
                    text: "buffer with same name is defined here".into(),
                },
            ],
            ast,
        )
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
    pub value: shad_parser::AstExpr,
    /// The buffer name in the initial Shad code.
    pub name: AstIdent,
}

impl Buffer {
    pub(crate) fn new(buffer: &AstBufferItem, index: usize, value_type: Rc<Type>) -> Self {
        Self {
            index,
            type_: value_type,
            value: buffer.value.clone(),
            name: buffer.name.clone(),
        }
    }
}
