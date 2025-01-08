use crate::resolving::items::Item;
use crate::{resolving, Analysis, BufferId};
use fxhash::FxHashSet;
use shad_parser::{AstFnCall, AstIdent, AstRunItem, Visit};

pub(crate) fn list_in_block(analysis: &Analysis, block: &AstRunItem) -> Vec<BufferId> {
    let mut listing = BufferListing::new(analysis);
    listing.visit_run_item(block);
    listing.buffers.into_iter().collect()
}

struct BufferListing<'a> {
    analysis: &'a Analysis,
    buffers: FxHashSet<BufferId>,
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
        if let Some(Item::Fn(fn_)) = resolving::items::item(self.analysis, &node.name) {
            self.visit_fn_item(&fn_.ast);
        }
    }

    fn enter_ident(&mut self, node: &AstIdent) {
        if let Some(Item::Buffer(buffer)) = resolving::items::item(self.analysis, node) {
            self.buffers.insert(buffer.id.clone());
        }
    }
}
