use crate::compilation::index::NodeIndex;
use crate::compilation::node::{choice, sequence, EndOfFile, Node, NodeConfig, Repeated};
use crate::compilation::transpilation::TranspilationContext;
use crate::compilation::validation::ValidationContext;
use crate::language::items::buffer::BufferItem;
use crate::language::items::compute::{InitItem, RunItem};
use crate::language::items::fn_::{FnItem, NativeFnItem};
use crate::language::items::import::ImportItem;
use crate::ValidationError;
use itertools::Itertools;
use std::any::TypeId;

pub(crate) mod block;
pub(crate) mod buffer;
pub(crate) mod compute;
pub(crate) mod fn_;
pub(crate) mod import;
pub(crate) mod type_;

sequence!(
    struct Root {
        items: Repeated<Item, 0, { usize::MAX }>,
        #[force_error(true)]
        eof: EndOfFile,
    }
);

impl NodeConfig for Root {}

choice!(
    enum Item {
        Import(ImportItem),
        Buffer(BufferItem),
        Init(InitItem),
        Run(RunItem),
        NativeFn(NativeFnItem),
        Fn(FnItem),
    }
);

impl NodeConfig for Item {}

fn check_duplicated_items(item: &impl Node, ctx: &mut ValidationContext<'_>) {
    let key = item
        .key()
        .expect("internal error: cannot calculate item key");
    for other_item in ctx.roots[&item.path].items.iter() {
        let other_item = other_item.inner();
        if other_item.id < item.id && other_item.key().as_ref() == Some(&key) {
            ctx.errors.push(ValidationError::error(
                ctx,
                item,
                &format!("{key} defined multiple times"),
                Some("duplicated item"),
                &[(other_item, "same item defined here")],
            ));
        }
    }
}

fn is_item_recursive(item: &impl Node, index: &NodeIndex) -> bool {
    item.nested_sources(index)
        .iter()
        .any(|source| source.id == item.id)
}

fn check_recursive_items(item: &impl Node, ctx: &mut ValidationContext<'_>) {
    if is_item_recursive(item, ctx.index) {
        ctx.errors.push(ValidationError::error(
            ctx,
            item,
            "item definition with circular dependency",
            Some("this item is directly or indirectly referring to itself"),
            &[],
        ));
    }
}

fn transpiled_dependencies(ctx: &mut TranspilationContext<'_>, item: &impl Node) -> String {
    item.nested_sources(ctx.index)
        .into_iter()
        .filter(|source| {
            [TypeId::of::<BufferItem>(), TypeId::of::<FnItem>()].contains(&(*source).node_type_id())
        })
        .map(|source| source.transpile(ctx))
        .join("\n")
}
