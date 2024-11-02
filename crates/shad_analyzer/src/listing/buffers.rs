use crate::registration::idents::IdentSource;
use crate::Analysis;
use fxhash::FxHashSet;
use shad_parser::{AstFnCall, AstIdent, AstRunItem, Visit};

pub(crate) fn list(analysis: &Analysis, block: &AstRunItem) -> Vec<String> {
    let mut listing = BufferListing::new(analysis);
    listing.visit_run_item(block);
    listing.buffers.into_iter().collect()
}

struct BufferListing<'a> {
    analysis: &'a Analysis,
    buffers: FxHashSet<String>,
}

impl<'a> BufferListing<'a> {
    fn new(analysis: &'a Analysis) -> Self {
        Self {
            analysis,
            buffers: FxHashSet::default(),
        }
    }
}

impl Visit for BufferListing<'_> {
    fn enter_fn_call(&mut self, node: &AstFnCall) {
        if let Some(signature) = self.analysis.fn_signature(&node.name) {
            self.visit_fn_item(&self.analysis.fns[&signature].ast);
        }
    }

    fn enter_ident(&mut self, node: &AstIdent) {
        let ident = self
            .analysis
            .idents
            .get(&node.id)
            .expect("internal error: missing identifier ID");
        match &ident.source {
            IdentSource::Buffer(name) => {
                self.buffers.insert(name.clone());
            }
            IdentSource::Ident(_) | IdentSource::Fn(_) => (),
        }
    }
}
