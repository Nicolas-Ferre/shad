use crate::compilation::constant::{
    ConstantContext, ConstantData, ConstantStructFieldData, ConstantValue,
};
use crate::compilation::index::NodeIndex;
use crate::compilation::node::{sequence, Node, NodeConfig, NodeType, Repeated};
use crate::compilation::transpilation::TranspilationContext;
use crate::compilation::validation::ValidationContext;
use crate::language::expressions::fn_call::{FnArg, FnArgGroup};
use crate::language::keywords::{CloseCurlyBracketSymbol, OpenCurlyBracketSymbol};
use crate::language::type_ref::Type;
use crate::language::{sources, validations};
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

    fn source<'a>(&'a self, index: &'a NodeIndex) -> Option<&'a dyn Node> {
        index.search(self, &self.source_key(index)?, sources::type_criteria())
    }

    fn is_ref(&self, _index: &NodeIndex) -> Option<bool> {
        Some(false)
    }

    fn type_<'a>(&'a self, index: &'a NodeIndex) -> Option<NodeType<'a>> {
        self.type_.type_(index)
    }

    fn validate(&self, ctx: &mut ValidationContext<'_>) {
        if let Some(type_item) = self.type_.item(ctx.index) {
            if type_item.is_native() {
                ctx.errors.push(ValidationError::error(
                    ctx,
                    self,
                    "cannot call constructor for a native type",
                    Some("constructor called here"),
                    &[],
                ));
                return;
            }
            if type_item.path != self.path
                && type_item.fields().iter().any(|field| !field.is_public())
            {
                ctx.errors.push(ValidationError::error(
                    ctx,
                    self,
                    "cannot call constructor for a type with at least one private field",
                    Some("constructor called here"),
                    &[],
                ));
                return;
            }
            let fields = type_item.fields();
            let expected_field_count = fields.len();
            let actual_field_count = self.args().count();
            if expected_field_count == actual_field_count {
                for (arg, field) in self.args().zip(fields) {
                    let arg_name = arg.name.iter().next().map(|name| &*name.ident);
                    validations::check_invalid_expr_type(field, arg, true, ctx);
                    validations::check_arg_name(arg_name, &field.ident, ctx);
                }
            } else {
                ctx.errors.push(ValidationError::error(
                    ctx,
                    self,
                    "invalid number of fields",
                    Some(&format!("{actual_field_count} fields specified here")),
                    &[(
                        type_item,
                        &format!("{expected_field_count} fields expected"),
                    )],
                ));
            }
        }
    }

    fn invalid_constant(&self, index: &NodeIndex) -> Option<&dyn Node> {
        self.args().find_map(|arg| arg.invalid_constant(index))
    }

    fn evaluate_constant(&self, ctx: &mut ConstantContext<'_>) -> Option<ConstantValue> {
        let type_ = self.type_.type_(ctx.index)?;
        Some(ConstantValue {
            transpiled_type_name: type_.transpiled_name(ctx.index),
            data: ConstantData::StructFields(
                type_
                    .source()
                    .expect("internal error: type reference shouldn't be <no return>")
                    .item
                    .fields()
                    .iter()
                    .zip(self.args())
                    .map(|(field, arg)| ConstantStructFieldData {
                        name: field.ident.slice.clone(),
                        value: arg
                            .evaluate_constant(ctx)
                            .expect("internal error: invalid const constructor arg"),
                        is_alias: false,
                    })
                    .collect(),
            ),
        })
    }

    fn transpile(&self, ctx: &mut TranspilationContext<'_>) -> String {
        let type_name = self
            .type_
            .type_(ctx.index)
            .expect("internal error: constructor source not found")
            .transpiled_name(ctx.index);
        let args = self.args().map(|arg| arg.transpile(ctx)).join(", ");
        format!("{type_name}({args})")
    }
}

impl ConstructorExpr {
    fn args(&self) -> impl Iterator<Item = &FnArg> {
        self.args
            .iter()
            .flat_map(|args| args.args().map(|arg| &**arg))
    }
}
