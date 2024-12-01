use crate::registration::idents::IdentSource;
use crate::{resolver, Analysis, FnId};
use fxhash::FxHashSet;
use shad_parser::{AstFnCall, AstIdent, AstRunItem, AstStatement, Visit};

pub(crate) fn list_in_block(analysis: &Analysis, block: &AstRunItem) -> Vec<FnId> {
    let mut listing = FunctionListing::new(analysis);
    listing.visit_run_item(block);
    listing.fn_ids.into_iter().collect()
}

pub(crate) fn list_in_statement(analysis: &Analysis, statement: &AstStatement) -> Vec<FnId> {
    let mut listing = FunctionListing::new(analysis);
    listing.visit_statement(statement);
    listing.fn_ids.into_iter().collect()
}

struct FunctionListing<'a> {
    analysis: &'a Analysis,
    fn_ids: FxHashSet<FnId>,
}

impl<'a> FunctionListing<'a> {
    fn new(analysis: &'a Analysis) -> Self {
        Self {
            analysis,
            fn_ids: FxHashSet::default(),
        }
    }
}

impl Visit for FunctionListing<'_> {
    fn enter_fn_call(&mut self, node: &AstFnCall) {
        if let Some(fn_) = resolver::fn_(self.analysis, &node.name) {
            self.visit_fn_item(&fn_.ast);
        }
    }

    fn enter_ident(&mut self, node: &AstIdent) {
        let ident = self
            .analysis
            .idents
            .get(&node.id)
            .expect("internal error: missing identifier ID");
        match &ident.source {
            IdentSource::Fn(id) => {
                self.fn_ids.insert(id.clone());
            }
            IdentSource::Var(_) | IdentSource::Buffer(_) => (),
        }
    }
}
