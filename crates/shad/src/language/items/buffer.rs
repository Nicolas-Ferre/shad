use crate::compilation::index::NodeIndex;
use crate::compilation::node::{sequence, NodeConfig};
use crate::compilation::transpilation::TranspilationContext;
use crate::compilation::validation::ValidationContext;
use crate::language::expressions::TypedExpr;
use crate::language::items::{
    check_duplicated_items, check_recursive_items, is_item_recursive, transpiled_dependencies,
    type_,
};
use crate::language::keywords::{BufKeyword, EqSymbol, SemicolonSymbol};
use crate::language::patterns::Ident;
use crate::language::sources;
use indoc::indoc;
use itertools::Itertools;
use std::path::Path;

sequence!(
    struct BufferItem {
        buf: BufKeyword,
        #[force_error(true)]
        ident: Ident,
        eq: EqSymbol,
        expr: TypedExpr,
        semicolon: SemicolonSymbol,
    }
);

impl NodeConfig for BufferItem {
    fn key(&self) -> Option<String> {
        Some(sources::variable_key(&self.ident))
    }

    fn expr_type(&self, index: &NodeIndex) -> Option<String> {
        if is_item_recursive(self, index) {
            None
        } else {
            self.expr.expr_type(index)
        }
    }

    fn validate(&self, ctx: &mut ValidationContext<'_>) {
        check_duplicated_items(self, ctx);
        check_recursive_items(self, ctx);
    }

    fn transpile(&self, ctx: &mut TranspilationContext<'_>) -> String {
        format!(
            indoc!(
                "@group(0) @binding({next_binding})
                var<storage, read_write> _{id}: {type_};"
            ),
            id = self.id,
            type_ = type_::transpile_type(
                self.expr_type(ctx.index)
                    .expect("internal error: cannot calculate buffer type")
            ),
            next_binding = ctx.next_binding(),
        )
    }
}

impl BufferItem {
    pub(crate) fn transpile_shader(&self, ctx: &mut TranspilationContext<'_>) -> String {
        format!(
            indoc!(
                "{dependencies}
                {self_}

                @compute
                @workgroup_size(1, 1, 1)
                fn main() {{
                    _{id} = {expr};
                }}"
            ),
            id = self.id,
            expr = self.expr.transpile(ctx),
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
}
