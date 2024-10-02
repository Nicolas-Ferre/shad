use crate::statement::AsgStatements;
use crate::{Asg, AsgExpr, Result};
use shad_parser::AstBufferItem;

/// An analyzed buffer.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AsgBuffer {
    /// The parsed buffer.
    pub ast: AstBufferItem,
    /// The unique buffer index.
    pub index: usize,
    /// The initial value of the buffer.
    pub expr: Result<AsgExpr>,
}

impl AsgBuffer {
    pub(crate) fn new(asg: &mut Asg, ctx: &AsgStatements<'_>, buffer: &AstBufferItem) -> Self {
        Self {
            ast: buffer.clone(),
            index: asg.buffers.len(),
            expr: AsgExpr::new(asg, ctx, &buffer.value),
        }
    }
}
