use crate::compilation::node::{sequence, NodeConfig};
use crate::compilation::transpilation::TranspilationContext;
use crate::language::items::block::NonReturnBlock;
use crate::language::items::transpiled_dependencies;
use crate::language::keywords::{InitKeyword, RunKeyword};
use indoc::indoc;

sequence!(
    struct InitItem {
        init: InitKeyword,
        #[force_error(true)]
        block: NonReturnBlock,
    }
);

impl NodeConfig for InitItem {}

impl InitItem {
    pub(crate) fn transpile_shader(&self, ctx: &mut TranspilationContext<'_>) -> String {
        format!(
            indoc!(
                "{dependencies}

                @compute
                @workgroup_size(1, 1, 1)
                fn main() {{
                {block}
                }}"
            ),
            dependencies = transpiled_dependencies(ctx, self),
            block = self.block.transpile(ctx),
        )
    }
}

sequence!(
    struct RunItem {
        run: RunKeyword,
        #[force_error(true)]
        block: NonReturnBlock,
    }
);

impl NodeConfig for RunItem {}

impl RunItem {
    pub(crate) fn transpile_shader(&self, ctx: &mut TranspilationContext<'_>) -> String {
        format!(
            indoc!(
                "{dependencies}

                @compute
                @workgroup_size(1, 1, 1)
                fn main() {{
                {block}
                }}"
            ),
            dependencies = transpiled_dependencies(ctx, self),
            block = self.block.transpile(ctx),
        )
    }
}
