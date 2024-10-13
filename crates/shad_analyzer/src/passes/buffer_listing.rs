use crate::result::result_ref;
use crate::{
    Asg, AsgAssignment, AsgBuffer, AsgExpr, AsgFnCall, AsgIdent, AsgIdentSource, AsgLeftValue,
    AsgStatement, AsgVariableDefinition,
};
use fxhash::FxHashMap;
use std::rc::Rc;

/// A trait implemented to list buffers in a code fragment.
pub trait BufferListing: Sized {
    /// Returns the used buffers.
    fn buffers<'a>(&'a self, asg: &'a Asg) -> Vec<Rc<AsgBuffer>>;

    /// Returns the used buffers from a slice.
    fn slice_buffers(slice: &[Self], asg: &Asg) -> Vec<Rc<AsgBuffer>> {
        slice
            .iter()
            .flat_map(|statement| statement.buffers(asg))
            .map(|buffer| (buffer.index, buffer))
            .collect::<FxHashMap<_, _>>()
            .into_values()
            .collect()
    }
}

impl BufferListing for AsgStatement {
    fn buffers(&self, asg: &Asg) -> Vec<Rc<AsgBuffer>> {
        match self {
            Self::Assignment(statement) => statement.buffers(asg),
            Self::Var(statement) => statement.buffers(asg),
            Self::Return(statement) => statement
                .expr
                .as_ref()
                .map(|expr| expr.buffers(asg))
                .unwrap_or_default(),
            Self::FnCall(statement) => result_ref(statement)
                .map(|call| call.buffers(asg))
                .unwrap_or_default(),
        }
    }
}

impl BufferListing for AsgAssignment {
    fn buffers(&self, asg: &Asg) -> Vec<Rc<AsgBuffer>> {
        let assigned_buffers = result_ref(&self.assigned)
            .map(|value| value.buffers(asg))
            .unwrap_or_default();
        let expr_buffer = result_ref(&self.expr)
            .map(|expr| expr.buffers(asg))
            .unwrap_or_default();
        [assigned_buffers, expr_buffer].concat()
    }
}

impl BufferListing for AsgLeftValue {
    fn buffers(&self, asg: &Asg) -> Vec<Rc<AsgBuffer>> {
        match self {
            Self::Ident(ident) => ident.buffers(asg),
            Self::FnCall(call) => call.buffers(asg),
        }
    }
}

impl BufferListing for AsgVariableDefinition {
    fn buffers(&self, asg: &Asg) -> Vec<Rc<AsgBuffer>> {
        result_ref(&self.expr)
            .map(|expr| expr.buffers(asg))
            .unwrap_or_default()
    }
}

impl BufferListing for AsgExpr {
    fn buffers(&self, asg: &Asg) -> Vec<Rc<AsgBuffer>> {
        match self {
            Self::Literal(_) => vec![],
            Self::Ident(expr) => expr.buffers(asg),
            Self::FnCall(expr) => expr.buffers(asg),
        }
    }
}

impl BufferListing for AsgIdent {
    fn buffers(&self, _asg: &Asg) -> Vec<Rc<AsgBuffer>> {
        match &self.source {
            AsgIdentSource::Buffer(buffer) => vec![buffer.clone()],
            AsgIdentSource::Var(_) | AsgIdentSource::Param(_) => vec![],
        }
    }
}

impl BufferListing for AsgFnCall {
    fn buffers(&self, asg: &Asg) -> Vec<Rc<AsgBuffer>> {
        let arg_buffers = self.args.iter().flat_map(|arg| arg.buffers(asg));
        let body_buffers = asg.function_bodies[self.fn_.index]
            .statements
            .iter()
            .flat_map(|statement| statement.buffers(asg));
        arg_buffers.chain(body_buffers).collect()
    }
}
