use crate::{listing, Function};
use crate::{Analysis, TypeId};
use fxhash::FxHashSet;
use shad_parser::AstRunItem;
use std::iter;

pub(crate) fn list_in_block(analysis: &Analysis, block: &AstRunItem) -> Vec<TypeId> {
    let fn_types = listing::functions::list_in_block(analysis, block)
        .into_iter()
        .map(|fn_id| &analysis.fns[&fn_id])
        .flat_map(fn_type_ids);
    let buffer_types = listing::buffers::list_in_block(analysis, block)
        .into_iter()
        .map(|buffer_id| &analysis.buffers[&buffer_id])
        .filter_map(|buffer| analysis.idents[&buffer.ast.name.id].type_id.clone());
    fn_types
        .chain(buffer_types)
        .flat_map(|type_id| all_recursive_type_ids(analysis, type_id))
        .collect::<FxHashSet<_>>()
        .into_iter()
        .collect()
}

fn fn_type_ids(fn_: &Function) -> impl Iterator<Item = TypeId> + '_ {
    let param_types = fn_.params.iter().filter_map(|param| param.type_id.clone());
    let return_type = fn_.return_type_id.iter().map(Clone::clone);
    param_types.chain(return_type)
}

fn all_recursive_type_ids(analysis: &Analysis, type_id: TypeId) -> Vec<TypeId> {
    let type_ = &analysis.types[&type_id];
    if type_.fields.is_empty() {
        vec![type_id]
    } else {
        let child_type_ids = type_
            .fields
            .iter()
            .filter_map(|field| field.type_id.clone())
            .flat_map(|type_id| all_recursive_type_ids(analysis, type_id));
        iter::once(type_id).chain(child_type_ids).collect()
    }
}
