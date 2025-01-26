use crate::{resolving, Analysis, FnId, GenericValue};
use fxhash::FxHashSet;
use shad_parser::{AstFnCall, AstRunItem, AstStatement, Visit};

/// A specialized function.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SpecializedFn {
    /// The source function ID.
    pub id: FnId,
    /// The generic arguments passed to the function.
    pub generic_args: Vec<(String, GenericValue)>,
}

pub(crate) fn list_in_block(analysis: &Analysis, block: &AstRunItem) -> Vec<SpecializedFn> {
    let mut listing = FunctionListing::new(analysis);
    listing.visit_run_item(block);
    listing.fn_ids.into_iter().collect()
}

pub(crate) fn list_in_statement(
    analysis: &Analysis,
    statement: &AstStatement,
) -> Vec<SpecializedFn> {
    let mut listing = FunctionListing::new(analysis);
    listing.visit_statement(statement);
    listing.fn_ids.into_iter().collect()
}

struct FunctionListing<'a> {
    analysis: &'a Analysis,
    fn_ids: FxHashSet<SpecializedFn>,
    generic_values: Vec<Vec<(String, GenericValue)>>,
}

impl<'a> FunctionListing<'a> {
    fn new(analysis: &'a Analysis) -> Self {
        Self {
            analysis,
            fn_ids: FxHashSet::default(),
            generic_values: vec![vec![]],
        }
    }
}

impl Visit for FunctionListing<'_> {
    fn enter_fn_call(&mut self, node: &AstFnCall) {
        if let Some(fn_) = resolving::items::fn_(self.analysis, node, true) {
            let generic_values = &self.generic_values[self.generic_values.len() - 1];
            let generic_args =
                resolving::expressions::fn_call_generic_values(self.analysis, node, generic_values)
                    .into_iter()
                    .zip(&fn_.generics)
                    .map(|(value, param)| (param.name().label.clone(), value))
                    .collect::<Vec<_>>();
            self.fn_ids.insert(SpecializedFn {
                id: fn_.id.clone(),
                generic_args: generic_args.clone(),
            });
            self.generic_values.push(generic_args);
            self.visit_fn_item(&fn_.ast);
            self.generic_values.pop();
        }
    }
}
