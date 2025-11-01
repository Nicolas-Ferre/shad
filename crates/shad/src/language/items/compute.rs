use crate::compilation::constant::{ConstantContext, ConstantData, ConstantValue};
use crate::compilation::index::NodeIndex;
use crate::compilation::node::{sequence, NodeConfig, NodeType, Repeated};
use crate::compilation::transpilation::TranspilationContext;
use crate::compilation::validation::ValidationContext;
use crate::language::expressions::binary::MaybeBinaryExpr;
use crate::language::items::block::NonReturnBlock;
use crate::language::items::transpiled_dependencies;
use crate::language::keywords::{
    CloseParenthesisSymbol, InitKeyword, OpenParenthesisSymbol, PrioKeyword, RunKeyword,
};
use crate::language::patterns::I32Literal;
use crate::language::validations;
use indoc::indoc;

sequence!(
    struct InitItem {
        init: InitKeyword,
        #[force_error(true)]
        priority: Repeated<Priority, 0, 1>,
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

    pub(crate) fn priority(&self, index: &NodeIndex) -> i32 {
        self.priority
            .iter()
            .map(|priority| priority.value(index))
            .next()
            .unwrap_or(0)
    }
}

sequence!(
    struct RunItem {
        run: RunKeyword,
        #[force_error(true)]
        priority: Repeated<Priority, 0, 1>,
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

    pub(crate) fn priority(&self, index: &NodeIndex) -> i32 {
        self.priority
            .iter()
            .map(|priority| priority.value(index))
            .next()
            .unwrap_or(0)
    }
}

sequence!(
    struct Priority {
        prio: PrioKeyword,
        #[force_error(true)]
        args_start: OpenParenthesisSymbol,
        value: MaybeBinaryExpr,
        args_end: CloseParenthesisSymbol,
    }
);

impl NodeConfig for Priority {
    fn validate(&self, ctx: &mut ValidationContext<'_>) {
        let expected_type = NodeType::Source(I32Literal::i32_type(self, ctx.index));
        validations::check_invalid_const_expr_type(expected_type, &*self.value, ctx);
        validations::check_invalid_const_scope(&*self.value, &*self.prio, ctx);
    }
}

impl Priority {
    fn value(&self, index: &NodeIndex) -> i32 {
        let mut ctx = ConstantContext::new(index);
        if let Some(ConstantValue {
            data: ConstantData::I32(value),
            ..
        }) = self.value.evaluate_constant(&mut ctx)
        {
            value
        } else {
            unreachable!("priority should be a `const` expression");
        }
    }
}
