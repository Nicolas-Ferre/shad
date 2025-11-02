use crate::compilation::index::NodeIndex;
use crate::compilation::node::{choice, sequence, EndOfFile, Node, NodeConfig, Repeated};
use crate::compilation::transpilation::TranspilationContext;
use crate::language::items::buffer::BufferItem;
use crate::language::items::compute::{InitItem, RunItem};
use crate::language::items::constant::ConstantItem;
use crate::language::items::fn_::{FnItem, NativeFnItem};
use crate::language::items::import::ImportItem;
use crate::language::items::type_::{NativeStructItem, StructItem};
use itertools::Itertools;

pub(crate) mod block;
pub(crate) mod buffer;
pub(crate) mod compute;
pub(crate) mod constant;
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
        Constant(ConstantItem),
        Init(InitItem),
        Run(RunItem),
        NativeFn(NativeFnItem),
        Fn(FnItem),
        NativeStruct(NativeStructItem),
        Struct(StructItem),
    }
);

pub(crate) fn is_item_recursive(item: &impl Node, index: &NodeIndex) -> bool {
    item.nested_sources(index)
        .iter()
        .any(|source| source.id == item.id)
}

fn transpiled_dependencies(ctx: &mut TranspilationContext<'_>, item: &impl Node) -> String {
    item.nested_sources(ctx.index)
        .into_iter()
        .filter(|source| source.is_transpilable_dependency(ctx.index))
        .map(|source| source.transpile(ctx))
        .join("\n")
}
