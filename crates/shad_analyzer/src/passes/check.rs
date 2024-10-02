use crate::{Asg, AsgFn};
use shad_error::SemanticError;

pub(crate) trait ErrorCheck {
    fn check(&self, asg: &Asg, ctx: &mut ErrorCheckContext) -> Vec<SemanticError>;
}

struct ErrorCheckContext {}

impl ErrorCheck for Asg {
    fn check(&self, asg: &Asg, ctx: &mut ErrorCheckContext) -> Vec<SemanticError> {
        todo!()
    }
}

impl ErrorCheck for AsgFn {
    fn check(&self, asg: &Asg, ctx: &mut ErrorCheckContext) -> Vec<SemanticError> {
        todo!()
    }
}
