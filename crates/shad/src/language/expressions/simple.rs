use crate::compilation::constant::{ConstantContext, ConstantData, ConstantValue};
use crate::compilation::index::NodeIndex;
use crate::compilation::node::{
    choice, sequence, GenericArgs, Node, NodeConfig, NodeRef, NodeSource,
};
use crate::compilation::transpilation::TranspilationContext;
use crate::compilation::validation::ValidationContext;
use crate::compilation::PRELUDE_PATH;
use crate::language::expressions::binary::MaybeBinaryExpr;
use crate::language::items::constant::ConstantItem;
use crate::language::items::fn_::FnParam;
use crate::language::items::type_;
use crate::language::items::type_::TypeItem;
use crate::language::keywords::{
    AlignofKeyword, CloseParenthesisSymbol, FalseKeyword, OpenParenthesisSymbol, SizeofKeyword,
    TrueKeyword,
};
use crate::language::patterns::{Ident, U32Literal};
use crate::language::sources;
use crate::language::statements::{LocalRefDefStmt, LocalVarDefStmt};
use crate::language::type_ref::Type;
use crate::language::validations;
use std::any::TypeId;
use std::path::Path;

sequence!(
    #[allow(unused_mut)]
    struct TrueLiteral {
        keyword: TrueKeyword,
    }
);

impl NodeConfig for TrueLiteral {
    fn is_ref(&self, _index: &NodeIndex) -> Option<bool> {
        Some(false)
    }

    fn type_<'a>(&'a self, index: &'a NodeIndex) -> Option<NodeSource<'a>> {
        Some(NodeSource {
            node: NodeRef::Type(bool_type(self, index)),
            generic_args: vec![],
        })
    }

    fn invalid_constant(&self, _index: &NodeIndex) -> Option<&dyn Node> {
        None
    }

    fn evaluate_constant(&self, ctx: &mut ConstantContext<'_>) -> Option<ConstantValue> {
        Some(ConstantValue {
            transpiled_type_name: bool_type(self, ctx.index).transpiled_name(ctx.index, &vec![]),
            data: ConstantData::Bool(true),
        })
    }

    fn transpile(
        &self,
        _ctx: &mut TranspilationContext<'_>,
        _generic_args: &GenericArgs<'_>,
    ) -> String {
        "u32(true)".into()
    }
}

sequence!(
    #[allow(unused_mut)]
    struct FalseLiteral {
        keyword: FalseKeyword,
    }
);

impl NodeConfig for FalseLiteral {
    fn is_ref(&self, _index: &NodeIndex) -> Option<bool> {
        Some(false)
    }

    fn type_<'a>(&'a self, index: &'a NodeIndex) -> Option<NodeSource<'a>> {
        Some(NodeSource {
            node: NodeRef::Type(bool_type(self, index)),
            generic_args: vec![],
        })
    }

    fn invalid_constant(&self, _index: &NodeIndex) -> Option<&dyn Node> {
        None
    }

    fn evaluate_constant(&self, ctx: &mut ConstantContext<'_>) -> Option<ConstantValue> {
        Some(ConstantValue {
            transpiled_type_name: bool_type(self, ctx.index).transpiled_name(ctx.index, &vec![]),
            data: ConstantData::Bool(false),
        })
    }

    fn transpile(
        &self,
        _ctx: &mut TranspilationContext<'_>,
        _generic_args: &GenericArgs<'_>,
    ) -> String {
        "u32(false)".into()
    }
}

fn bool_type<'a>(node: &impl Node, index: &'a NodeIndex) -> &'a dyn TypeItem {
    type_::to_item(
        index
            .search_in_path(
                Path::new(PRELUDE_PATH),
                node,
                "`bool` type",
                sources::type_criteria(),
            )
            .expect("internal error: `bool` type not found"),
    )
}

sequence!(
    #[allow(unused_mut)]
    struct VarIdentExpr {
        ident: Ident,
    }
);

impl NodeConfig for VarIdentExpr {
    fn source_key(&self, _index: &NodeIndex) -> Option<String> {
        Some(sources::variable_key(&self.ident))
    }

    fn source<'a>(&'a self, index: &'a NodeIndex) -> Option<NodeSource<'a>> {
        let source = index.search(self, &self.source_key(index)?, sources::variable_criteria())?;
        Some(NodeSource {
            node: NodeRef::Other(source),
            generic_args: vec![],
        })
    }

    fn is_ref(&self, index: &NodeIndex) -> Option<bool> {
        self.source(index)
            .map(|source| source.as_node().node_type_id() != TypeId::of::<ConstantItem>())
    }

    fn type_<'a>(&'a self, index: &'a NodeIndex) -> Option<NodeSource<'a>> {
        self.source(index)?.as_node().type_(index)
    }

    fn validate(&self, ctx: &mut ValidationContext<'_>) {
        validations::check_missing_source(self, ctx);
    }

    fn invalid_constant(&self, index: &NodeIndex) -> Option<&dyn Node> {
        let source = self.source(index)?.as_node();
        if source.node_type_id() == TypeId::of::<ConstantItem>()
            || source.node_type_id() == TypeId::of::<LocalVarDefStmt>()
            || source.node_type_id() == TypeId::of::<LocalRefDefStmt>()
            || source.node_type_id() == TypeId::of::<FnParam>()
        {
            None
        } else {
            Some(self)
        }
    }

    fn evaluate_constant(&self, ctx: &mut ConstantContext<'_>) -> Option<ConstantValue> {
        let var_def = self.source(ctx.index)?.as_node();
        if var_def.node_type_id() == TypeId::of::<ConstantItem>() {
            var_def.evaluate_constant(ctx)
        } else {
            ctx.var_value(var_def.id).cloned()
        }
    }

    fn transpile(
        &self,
        ctx: &mut TranspilationContext<'_>,
        _generic_args: &GenericArgs<'_>,
    ) -> String {
        let source_id = self
            .source(ctx.index)
            .expect("internal error: var ident source not found")
            .as_node()
            .id;
        if let Some(mapping) = ctx.inline_mapping(source_id) {
            mapping.to_string()
        } else {
            format!("_{source_id}")
        }
    }
}

sequence!(
    struct ParenthesizedExpr {
        open: OpenParenthesisSymbol,
        #[force_error(true)]
        expr: MaybeBinaryExpr,
        close: CloseParenthesisSymbol,
    }
);

impl NodeConfig for ParenthesizedExpr {
    fn source<'a>(&'a self, index: &'a NodeIndex) -> Option<NodeSource<'a>> {
        self.expr.source(index)
    }

    fn is_ref(&self, _index: &NodeIndex) -> Option<bool> {
        Some(false)
    }

    fn type_<'a>(&'a self, index: &'a NodeIndex) -> Option<NodeSource<'a>> {
        self.expr.type_(index)
    }

    fn invalid_constant(&self, index: &NodeIndex) -> Option<&dyn Node> {
        self.expr.invalid_constant(index)
    }

    fn evaluate_constant(&self, ctx: &mut ConstantContext<'_>) -> Option<ConstantValue> {
        self.expr.evaluate_constant(ctx)
    }

    fn transpile(
        &self,
        ctx: &mut TranspilationContext<'_>,
        generic_args: &GenericArgs<'_>,
    ) -> String {
        let expr = self.expr.transpile(ctx, generic_args);
        format!("({expr})")
    }
}

sequence!(
    struct TypeOperationExpr {
        operator: TypeOperator,
        #[force_error(true)]
        start: OpenParenthesisSymbol,
        type_: Type,
        end: CloseParenthesisSymbol,
    }
);

impl NodeConfig for TypeOperationExpr {
    fn is_ref(&self, _index: &NodeIndex) -> Option<bool> {
        Some(false)
    }

    fn type_<'a>(&'a self, index: &'a NodeIndex) -> Option<NodeSource<'a>> {
        Some(NodeSource {
            node: NodeRef::Type(U32Literal::u32_type(self, index)),
            generic_args: vec![],
        })
    }

    fn invalid_constant(&self, _index: &NodeIndex) -> Option<&dyn Node> {
        None
    }

    fn evaluate_constant(&self, ctx: &mut ConstantContext<'_>) -> Option<ConstantValue> {
        Some(ConstantValue {
            transpiled_type_name: "u32".to_string(),
            data: ConstantData::U32(self.value(ctx.index)?),
        })
    }

    fn transpile(
        &self,
        ctx: &mut TranspilationContext<'_>,
        _generic_args: &GenericArgs<'_>,
    ) -> String {
        let value = self
            .value(ctx.index)
            .expect("internal error: cannot run type operation on an invalid type");
        format!("{value}u")
    }
}

impl TypeOperationExpr {
    fn value(&self, index: &NodeIndex) -> Option<u32> {
        let type_item = self.type_.item(index)?;
        Some(match *self.operator {
            TypeOperator::Alignof(_) => type_item.alignment(index),
            TypeOperator::Sizeof(_) => type_item.size(index),
        })
    }
}

choice!(
    enum TypeOperator {
        Alignof(AlignofKeyword),
        Sizeof(SizeofKeyword),
    }
);
