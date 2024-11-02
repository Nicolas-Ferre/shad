use crate::registration::idents::IdentSource;
use crate::Analysis;
use fxhash::FxHashSet;
use shad_parser::{AstFnCall, AstFnQualifier, AstIdent, AstRunItem, AstStatement, Visit};

pub(crate) fn list_in_block(analysis: &Analysis, block: &AstRunItem) -> Vec<String> {
    let mut listing = FunctionListing::new(analysis);
    listing.visit_run_item(block);
    listing.signatures.into_iter().collect()
}

pub(crate) fn list_in_statement(analysis: &Analysis, statement: &AstStatement) -> Vec<String> {
    let mut listing = FunctionListing::new(analysis);
    listing.visit_statement(statement);
    listing.signatures.into_iter().collect()
}

struct FunctionListing<'a> {
    analysis: &'a Analysis,
    signatures: FxHashSet<String>,
}

impl<'a> FunctionListing<'a> {
    fn new(analysis: &'a Analysis) -> Self {
        Self {
            analysis,
            signatures: FxHashSet::default(),
        }
    }
}

impl Visit for FunctionListing<'_> {
    fn enter_fn_call(&mut self, node: &AstFnCall) {
        if let Some(signature) = self.analysis.fn_signature(&node.name) {
            self.visit_fn_item(&self.analysis.fns[&signature].ast);
        }
    }

    fn enter_ident(&mut self, node: &AstIdent) {
        if let Some(ident) = self.analysis.idents.get(&node.id) {
            match &ident.source {
                IdentSource::Fn(signature) => {
                    if self.analysis.fns.get(signature).map_or(false, |fn_| {
                        !fn_.is_inlined() && fn_.ast.qualifier != AstFnQualifier::Gpu
                    }) {
                        self.signatures.insert(signature.clone());
                    }
                }
                IdentSource::Ident(_) | IdentSource::Buffer(_) => (),
            }
        }
    }
}
