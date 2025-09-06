use crate::compilation::index::NodeIndex;
use crate::compilation::node::{NodeConfig, NodeSourceSearchCriteria, Repeated};
use crate::language::expressions::binary::MaybeBinaryExpr;
use crate::language::items::buffer::BufferItem;
use crate::language::items::fn_::{FnItem, FnParam, FnParamGroup, NativeFnItem};
use crate::language::patterns::Ident;
use crate::language::statements::{LocalRefDefStmt, LocalVarDefStmt};
use itertools::Itertools;
use std::any::TypeId;

pub(crate) fn variable_key(ident: &Ident) -> String {
    let name = &ident.slice;
    format!("`{name}` variable")
}

pub(crate) fn fn_key_from_params(ident: &Ident, params: &Repeated<FnParamGroup, 0, 1>) -> String {
    let name = &ident.slice;
    let params = params
        .iter()
        .flat_map(|params| params.params())
        .map(|param| &param.type_.slice)
        .join(", ");
    format!("`{name}({params})` function")
}

pub(crate) fn fn_key_from_args<'a>(
    ident: &Ident,
    args: impl Iterator<Item = &'a MaybeBinaryExpr>,
    index: &NodeIndex,
) -> Option<String> {
    let name = &ident.slice;
    let arg_types = args
        .map(|arg| arg.expr_type(index))
        .collect::<Option<Vec<_>>>()?
        .join(", ");
    Some(format!("`{name}({arg_types})` function"))
}

pub(crate) fn fn_key_from_operator(
    name: &str,
    arg_types: impl IntoIterator<Item = String>,
) -> String {
    let arg_types = arg_types.into_iter().join(", ");
    format!("`{name}({arg_types})` function")
}

pub(crate) fn variable_criteria() -> &'static [NodeSourceSearchCriteria] {
    &[
        NodeSourceSearchCriteria {
            node_type: || TypeId::of::<LocalVarDefStmt>(),
            can_be_after: false,
            common_parent_count: None,
        },
        NodeSourceSearchCriteria {
            node_type: || TypeId::of::<LocalRefDefStmt>(),
            can_be_after: false,
            common_parent_count: None,
        },
        NodeSourceSearchCriteria {
            node_type: || TypeId::of::<FnParam>(),
            can_be_after: false,
            common_parent_count: Some(2),
        },
        NodeSourceSearchCriteria {
            node_type: || TypeId::of::<BufferItem>(),
            can_be_after: true,
            common_parent_count: None,
        },
    ]
}

pub(crate) fn fn_criteria() -> &'static [NodeSourceSearchCriteria] {
    &[
        NodeSourceSearchCriteria {
            node_type: || TypeId::of::<FnItem>(),
            can_be_after: true,
            common_parent_count: None,
        },
        NodeSourceSearchCriteria {
            node_type: || TypeId::of::<NativeFnItem>(),
            can_be_after: true,
            common_parent_count: None,
        },
    ]
}
