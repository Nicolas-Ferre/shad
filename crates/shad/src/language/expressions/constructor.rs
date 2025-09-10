use crate::compilation::index::NodeIndex;
use crate::compilation::node::{
    sequence, Node, NodeConfig, NodeSourceSearchCriteria, NodeType, Repeated,
};
use crate::compilation::transpilation::TranspilationContext;
use crate::compilation::validation::ValidationContext;
use crate::language::expressions::binary::MaybeBinaryExpr;
use crate::language::expressions::fn_call::FnArgGroup;
use crate::language::items::type_;
use crate::language::items::type_::Type;
use crate::language::keywords::{CloseCurlyBracketSymbol, OpenCurlyBracketSymbol};
use crate::language::sources::check_missing_source;
use crate::language::{expressions, sources};
use crate::ValidationError;
use itertools::Itertools;

sequence!(
    struct ConstructorExpr {
        type_: Type,
        args_start: OpenCurlyBracketSymbol,
        #[force_error(true)]
        args: Repeated<FnArgGroup, 0, 1>,
        args_end: CloseCurlyBracketSymbol,
    }
);

impl NodeConfig for ConstructorExpr {
    fn source_key(&self, _index: &NodeIndex) -> Option<String> {
        Some(sources::type_key(&self.type_.ident))
    }

    fn source<'a>(&self, index: &'a NodeIndex) -> Option<&'a dyn Node> {
        index.search(self, &self.source_key(index)?)
    }

    fn source_search_criteria(&self) -> &'static [NodeSourceSearchCriteria] {
        sources::type_criteria()
    }

    fn type_<'a>(&self, index: &'a NodeIndex) -> Option<NodeType<'a>> {
        Some(NodeType::Source(self.type_.source(index)?))
    }

    fn validate(&self, ctx: &mut ValidationContext<'_>) {
        check_missing_source(self, ctx);
        if let Some(source) = self.source(ctx.index) {
            let fields = type_::fields(source);
            let expected_field_count = fields.len();
            let actual_field_count = self.args().count();
            if expected_field_count == actual_field_count {
                for (arg, field) in self.args().zip(fields) {
                    expressions::check_invalid_expr_type(field, arg, true, ctx);
                }
            } else {
                ctx.errors.push(ValidationError::error(
                    ctx,
                    self,
                    "invalid number of fields",
                    Some(&format!("{actual_field_count} fields specified here")),
                    &[(source, &format!("{expected_field_count} fields expected"))],
                ));
            }
        }
    }

    fn transpile(&self, ctx: &mut TranspilationContext<'_>) -> String {
        let source = self
            .type_
            .source(ctx.index)
            .expect("internal error: constructor source not found");
        let type_name = type_::transpile_name(source);
        let args = self.args().map(|arg| arg.transpile(ctx)).join(", ");
        format!("{type_name}({args})")
    }
}

impl ConstructorExpr {
    fn args(&self) -> impl Iterator<Item = &MaybeBinaryExpr> {
        self.args
            .iter()
            .flat_map(|args| args.args().map(|arg| &**arg))
    }
}
