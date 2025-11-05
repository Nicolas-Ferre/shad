use crate::compilation::index::NodeIndex;
use crate::compilation::node::{sequence, NodeConfig, NodeType, Repeated};
use crate::compilation::transpilation::TranspilationContext;
use crate::compilation::validation::ValidationContext;
use crate::language::expressions::binary::MaybeBinaryExpr;
use crate::language::items::type_::TypeItem;
use crate::language::items::{is_item_recursive, transpiled_dependencies};
use crate::language::keywords::{BufKeyword, EqSymbol, PubKeyword, SemicolonSymbol};
use crate::language::patterns::Ident;
use crate::language::{sources, validations};
use indoc::indoc;
use itertools::Itertools;
use std::path::Path;

sequence!(
    struct BufferItem {
        pub_: Repeated<PubKeyword, 0, 1>,
        buf: BufKeyword,
        #[force_error(true)]
        ident: Ident,
        eq: EqSymbol,
        expr: MaybeBinaryExpr,
        semicolon: SemicolonSymbol,
    }
);

impl NodeConfig for BufferItem {
    fn key(&self) -> Option<String> {
        Some(sources::variable_key(&self.ident))
    }

    fn is_public(&self) -> bool {
        self.pub_.iter().len() > 0
    }

    fn type_<'a>(&'a self, index: &'a NodeIndex) -> Option<NodeType<'a>> {
        if is_item_recursive(self, index) {
            None
        } else {
            self.expr.type_(index)
        }
    }

    fn validate(&self, ctx: &mut ValidationContext<'_>) {
        validations::check_duplicated_items(self, ctx);
        validations::check_recursive_items(self, ctx);
        validations::check_no_return_type(&*self.expr, ctx);
    }

    fn is_transpilable_dependency(&self, _index: &NodeIndex) -> bool {
        true
    }

    fn transpile(&self, ctx: &mut TranspilationContext<'_>) -> String {
        format!(
            indoc!(
                "@group(0) @binding({next_binding})
                var<storage, read_write> _{id}: {type_};"
            ),
            id = self.id,
            type_ = self.buffer_type(ctx.index).transpiled_name(ctx.index),
            next_binding = ctx.next_binding(),
        )
    }
}

impl BufferItem {
    pub(crate) fn buffer_type_item<'a>(&'a self, index: &'a NodeIndex) -> &'a dyn TypeItem {
        self.buffer_type(index)
            .source()
            .expect("internal error: buffer has <no return> type")
            .item
    }

    pub(crate) fn transpile_shader(&self, ctx: &mut TranspilationContext<'_>) -> String {
        let expr = self.expr.transpile(ctx);
        format!(
            indoc!(
                "{dependencies}
                {self_}

                @compute
                @workgroup_size(1, 1, 1)
                fn main() {{
                    {stmts}
                    _{id} = {expr};
                }}"
            ),
            id = self.id,
            stmts = ctx.generated_stmts.join("\n"),
            expr = expr,
            dependencies = transpiled_dependencies(ctx, self),
            self_ = self.transpile(ctx),
        )
    }

    pub(crate) fn item_path(&self, root_path: &Path) -> String {
        format!(
            "{}.{}",
            self.path
                .strip_prefix(root_path)
                .expect("internal error: invalid root path")
                .with_extension("")
                .components()
                .map(|component| component.as_os_str().to_string_lossy())
                .join("."),
            self.ident.slice
        )
    }

    fn buffer_type<'a>(&'a self, index: &'a NodeIndex) -> NodeType<'a> {
        self.type_(index)
            .expect("internal error: buffer type not found")
    }
}
