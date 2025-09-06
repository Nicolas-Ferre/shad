use crate::compilation::node::{sequence, NodeConfig, Repeated};
use crate::compilation::transpilation::TranspilationContext;
use crate::compilation::validation::ValidationContext;
use crate::language::keywords::{CloseCurlyBracketSymbol, OpenCurlyBracketSymbol};
use crate::language::statements::Stmt;
use crate::ValidationError;
use itertools::Itertools;
use std::mem;

sequence!(
    #[allow(unused_mut)]
    struct NonReturnBlock {
        inner: Block,
    }
);

impl NodeConfig for NonReturnBlock {
    fn validate(&self, ctx: &mut ValidationContext<'_>) {
        if let Some(return_stmt) = self
            .inner
            .statements
            .iter()
            .find_map(|stmt| stmt.as_return())
        {
            ctx.errors.push(ValidationError::error(
                ctx,
                return_stmt,
                "`return` statement used outside a function",
                Some("not allowed statement"),
                &[],
            ));
        }
    }

    fn transpile(&self, ctx: &mut TranspilationContext<'_>) -> String {
        self.inner.transpile(ctx)
    }
}

sequence!(
    struct Block {
        open: OpenCurlyBracketSymbol,
        #[force_error(true)]
        statements: Repeated<Stmt, 0, { usize::MAX }>,
        close: CloseCurlyBracketSymbol,
    }
);

impl NodeConfig for Block {
    fn validate(&self, ctx: &mut ValidationContext<'_>) {
        let last_stmt_id = self.last_stmt().map_or(0, |stmt| stmt.inner().id);
        for stmt in self.statements.iter() {
            if let Some(return_stmt) = stmt.as_return() {
                if return_stmt.id != last_stmt_id {
                    ctx.errors.push(ValidationError::error(
                        ctx,
                        return_stmt,
                        "`return` statement before end of the block",
                        Some("only allowed at the end of a block"),
                        &[],
                    ));
                }
            }
        }
    }

    fn transpile(&self, ctx: &mut TranspilationContext<'_>) -> String {
        ctx.start_block();
        let transpilation = self
            .statements
            .iter()
            .flat_map(|stmt| {
                let stmt = stmt.transpile(ctx);
                mem::take(&mut ctx.generated_stmts)
                    .into_iter()
                    .chain([stmt])
            })
            .join("\n");
        ctx.end_block();
        transpilation
    }
}

impl Block {
    pub(crate) fn last_stmt(&self) -> Option<&Stmt> {
        self.statements.iter().last().map(|stmt| &**stmt)
    }
}
