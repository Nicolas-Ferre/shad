use crate::compilation::constant::{ConstantContext, ConstantData, ConstantValue};
use crate::compilation::index::NodeIndex;
use crate::compilation::node::{sequence, NodeConfig, NodeType};
use crate::compilation::transpilation::TranspilationContext;
use crate::compilation::validation::ValidationContext;
use crate::language::expressions::TypedExpr;
use crate::language::items::is_item_recursive;
use crate::language::keywords::{ConstKeyword, EqSymbol, SemicolonSymbol};
use crate::language::patterns::Ident;
use crate::language::{sources, validations};
use itertools::Itertools;

sequence!(
    struct ConstantItem {
        const_: ConstKeyword,
        ident: Ident,
        #[force_error(true)]
        eq: EqSymbol,
        expr: TypedExpr,
        semicolon: SemicolonSymbol,
    }
);

impl NodeConfig for ConstantItem {
    fn key(&self) -> Option<String> {
        Some(sources::variable_key(&self.ident))
    }

    fn type_<'a>(&self, index: &'a NodeIndex) -> Option<NodeType<'a>> {
        if is_item_recursive(self, index) {
            None
        } else {
            self.expr.type_(index)
        }
    }

    fn validate(&self, ctx: &mut ValidationContext<'_>) {
        validations::check_duplicated_items(self, ctx);
        validations::check_recursive_items(self, ctx);
        validations::check_invalid_const_scope(&*self.expr, &self.const_, ctx);
    }

    fn evaluate_constant(&self, ctx: &mut ConstantContext<'_>) -> Option<ConstantValue> {
        self.expr.evaluate_constant(ctx)
    }

    fn is_transpilable_dependency(&self, _index: &NodeIndex) -> bool {
        true
    }

    fn transpile(&self, ctx: &mut TranspilationContext<'_>) -> String {
        let value = self
            .expr
            .evaluate_constant(&mut ConstantContext::new(ctx.index))
            .expect("internal error: missing const value");
        let id = self.id;
        let value = transpile_constant_value(&value);
        format!("const _{id} = {value};")
    }
}

fn transpile_constant_value(value: &ConstantValue) -> String {
    let type_name = &value.transpiled_type_name;
    let data = match &value.data {
        ConstantData::F32(value) => format!("{value}"),
        ConstantData::I32(value) => format!("{value}"),
        ConstantData::U32(value) => format!("{value}"),
        ConstantData::Bool(value) => format!("{value}"),
        ConstantData::StructFields(value) => value
            .iter()
            .filter(|field| !field.is_alias)
            .map(|field| transpile_constant_value(&field.value))
            .join(", "),
    };
    format!("{type_name}({data})")
}
