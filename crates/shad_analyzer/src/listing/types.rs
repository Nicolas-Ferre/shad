use crate::{listing, Function};
use crate::{Analysis, TypeId};
use fxhash::FxHashSet;
use shad_parser::AstRunItem;

pub(crate) fn list_in_block(analysis: &Analysis, block: &AstRunItem) -> Vec<TypeId> {
    listing::functions::list_in_block(analysis, block)
        .into_iter()
        .map(|fn_id| &analysis.fns[&fn_id])
        .flat_map(fn_type_ids)
        .filter(|type_id| analysis.types[type_id].ast.is_some())
        .collect::<FxHashSet<_>>()
        .into_iter()
        .collect()
}

fn fn_type_ids(fn_: &Function) -> impl Iterator<Item = TypeId> + '_ {
    let param_types = fn_.params.iter().filter_map(|param| param.type_id.clone());
    let return_type = fn_.return_type_id.iter().filter_map(Clone::clone);
    param_types.chain(return_type)
}
