use crate::compilation::index::NodeIndex;
use crate::compilation::node::{sequence, Node, NodeConfig, NodeType, NodeTypeSource, Repeated};
use crate::compilation::transpilation::TranspilationContext;
use crate::compilation::validation::ValidationContext;
use crate::language::expressions::binary::MaybeBinaryExpr;
use crate::language::keywords::{
    CloseAngleBracketSymbol, CloseCurlyBracketSymbol, ColonSymbol, CommaSymbol, EqSymbol,
    NativeKeyword, OpenAngleBracketSymbol, OpenCurlyBracketSymbol, PubKeyword, StructKeyword,
    TypeKeyword,
};
use crate::language::patterns::{Ident, StringLiteral, U32Literal};
use crate::language::type_ref::{Type, TypeGenericArgs};
use crate::language::{sources, validations};
use crate::ValidationError;
use indoc::indoc;
use itertools::Itertools;
use std::any::Any;
use std::iter;

pub(crate) trait TypeItem: Node {
    fn is_native(&self) -> bool;

    fn ident(&self) -> &Ident;

    fn generic_params(&self) -> Vec<&GenericParam>;

    fn fields(&self) -> Vec<&StructField>;

    fn field(&self, field_name: &str) -> Option<&StructField>;

    fn size(&self, index: &NodeIndex) -> u32;

    fn alignment(&self, index: &NodeIndex) -> u32;

    fn transpiled_name(&self, generics: Option<&TypeGenericArgs>, index: &NodeIndex) -> String;

    fn transpiled_field_name(&self, field_name: &str) -> String;
}

sequence!(
    struct NativeStructItem {
        pub_: Repeated<PubKeyword, 0, 1>,
        native: NativeKeyword,
        struct_: StructKeyword,
        #[force_error(true)]
        ident: Ident,
        generics: Repeated<GenericParams, 0, 1>,
        eq: EqSymbol,
        transpilation: StringLiteral,
        comma1: CommaSymbol,
        alignment: MaybeBinaryExpr,
        comma2: CommaSymbol,
        size: MaybeBinaryExpr,
        fields_start: OpenCurlyBracketSymbol,
        fields: Repeated<StructFieldGroup, 0, 1>,
        fields_end: CloseCurlyBracketSymbol,
    }
);

impl NodeConfig for NativeStructItem {
    fn key(&self) -> Option<String> {
        Some(sources::type_key(&self.ident))
    }

    fn is_public(&self) -> bool {
        self.pub_.iter().len() > 0
    }

    fn validate(&self, ctx: &mut ValidationContext<'_>) {
        let u32_type = NodeType::Source(NodeTypeSource {
            item: U32Literal::u32_type(self, ctx.index),
            generics: None,
        });
        validations::check_duplicated_items(self, ctx);
        validations::check_recursive_items(self, ctx);
        validations::check_invalid_const_expr_type(u32_type, &*self.alignment, ctx);
        validations::check_invalid_const_scope(&*self.alignment, &*self.native, ctx);
        validations::check_invalid_const_expr_type(u32_type, &*self.size, ctx);
        validations::check_invalid_const_scope(&*self.size, &*self.native, ctx);
    }

    fn is_transpilable_dependency(&self, _index: &NodeIndex) -> bool {
        false
    }
}

impl TypeItem for NativeStructItem {
    fn is_native(&self) -> bool {
        true
    }

    fn ident(&self) -> &Ident {
        &self.ident
    }

    fn generic_params(&self) -> Vec<&GenericParam> {
        self.generics
            .iter()
            .flat_map(|params| params.params())
            .collect()
    }

    // coverage: off (never call in practice)
    fn fields(&self) -> Vec<&StructField> {
        self.fields
            .iter()
            .flat_map(|fields| fields.iter())
            .collect()
    }
    // coverage: on

    fn field(&self, field_name: &str) -> Option<&StructField> {
        self.fields
            .iter()
            .find_map(|fields| fields.field(field_name))
    }

    fn size(&self, index: &NodeIndex) -> u32 {
        self.size.parse_const_u32(index)
    }

    fn alignment(&self, index: &NodeIndex) -> u32 {
        self.alignment.parse_const_u32(index)
    }

    fn transpiled_name(&self, generics: Option<&TypeGenericArgs>, index: &NodeIndex) -> String {
        let mut transpilation = self.transpilation.as_str().to_string();
        let args = generics.iter().flat_map(|args| args.args());
        let params = self.generics.iter().flat_map(|param| param.params());
        for (arg, param) in args.zip(params) {
            let transpiled_arg = arg
                .type_(index)
                .expect("internal error: invalid generic type argument")
                .transpiled_name(index);
            let placeholder = format!("${}", &param.ident.slice);
            transpilation = transpilation.replace(&placeholder, &transpiled_arg);
        }
        transpilation
    }

    fn transpiled_field_name(&self, field_name: &str) -> String {
        field_name.into()
    }
}

sequence!(
    struct StructItem {
        pub_: Repeated<PubKeyword, 0, 1>,
        struct_: StructKeyword,
        #[force_error(true)]
        ident: Ident,
        generics: Repeated<GenericParams, 0, 1>,
        fields_start: OpenCurlyBracketSymbol,
        fields: StructFieldGroup,
        fields_end: CloseCurlyBracketSymbol,
    }
);

impl NodeConfig for StructItem {
    fn key(&self) -> Option<String> {
        Some(sources::type_key(&self.ident))
    }

    fn is_public(&self) -> bool {
        self.pub_.iter().len() > 0
    }

    fn validate(&self, ctx: &mut ValidationContext<'_>) {
        validations::check_duplicated_items(self, ctx);
        validations::check_recursive_items(self, ctx);
        for field in self.fields.iter() {
            for other_field in self.fields.iter() {
                if other_field.id < field.id && other_field.ident.slice == field.ident.slice {
                    ctx.errors.push(ValidationError::error(
                        ctx,
                        field,
                        "struct field defined multiple times",
                        Some("duplicated field"),
                        &[(other_field, "same field defined here")],
                    ));
                }
            }
        }
    }

    fn is_transpilable_dependency(&self, _index: &NodeIndex) -> bool {
        true
    }

    fn transpile(&self, ctx: &mut TranspilationContext<'_>) -> String {
        let id = self.id;
        let fields = self
            .fields
            .iter()
            .map(|field| field.transpile(ctx))
            .join("\n");
        format!(
            indoc!(
                "struct _{id} {{
                {fields}
                }}
                "
            ),
            id = id,
            fields = fields
        )
    }
}

impl TypeItem for StructItem {
    fn is_native(&self) -> bool {
        false
    }

    fn ident(&self) -> &Ident {
        &self.ident
    }

    fn generic_params(&self) -> Vec<&GenericParam> {
        self.generics
            .iter()
            .flat_map(|params| params.params())
            .collect()
    }

    fn fields(&self) -> Vec<&StructField> {
        self.fields.iter().collect()
    }

    fn field(&self, field_name: &str) -> Option<&StructField> {
        self.fields.field(field_name)
    }

    fn size(&self, index: &NodeIndex) -> u32 {
        let fields: Vec<_> = self.fields.iter().collect();
        let last_field_size = fields[fields.len() - 1].type_item(index).size(index);
        round_up(
            self.alignment(index),
            field_offset(&fields, index) + last_field_size,
        )
    }

    fn alignment(&self, index: &NodeIndex) -> u32 {
        self.fields
            .iter()
            .map(|field| field.type_item(index).alignment(index))
            .max()
            .expect("internal error: custom structs should have at least one field")
    }

    fn transpiled_name(&self, _generics: Option<&TypeGenericArgs>, _index: &NodeIndex) -> String {
        format!("_{}", self.id)
    }

    fn transpiled_field_name(&self, field_name: &str) -> String {
        let field = self
            .field(field_name)
            .expect("internal error: field not found");
        format!("_{}", field.id)
    }
}

sequence!(
    struct GenericParams {
        start: OpenAngleBracketSymbol,
        #[force_error(true)]
        first_param: GenericParam,
        other_params: Repeated<OtherGenericParam, 0, { usize::MAX }>,
        final_comma: Repeated<CommaSymbol, 0, 1>,
        end: CloseAngleBracketSymbol,
    }
);

impl NodeConfig for GenericParams {
    fn validate(&self, ctx: &mut ValidationContext<'_>) {
        for param1 in self.params() {
            for param2 in self.params() {
                if param1.id < param2.id && param1.ident.slice == param2.ident.slice {
                    ctx.errors.push(ValidationError::error(
                        ctx,
                        param2,
                        "generic parameter defined multiple times",
                        Some("duplicated generic parameter name"),
                        &[(param1, "same generic parameter name defined here")],
                    ));
                }
            }
        }
    }
}

impl GenericParams {
    fn params(&self) -> impl Iterator<Item = &GenericParam> {
        iter::once(&*self.first_param).chain(self.other_params.iter().map(|other| &*other.param))
    }
}

sequence!(
    struct GenericParam {
        #[force_error(true)]
        ident: Ident,
        colon: ColonSymbol,
        type_: TypeKeyword,
    }
);

impl NodeConfig for GenericParam {}

sequence!(
    #[allow(unused_mut)]
    struct OtherGenericParam {
        comma: CommaSymbol,
        param: GenericParam,
    }
);

impl NodeConfig for OtherGenericParam {}

sequence!(
    struct StructFieldGroup {
        first_field: StructField,
        #[force_error(true)]
        other_fields: Repeated<StructOtherField, 0, { usize::MAX }>,
        final_comma: Repeated<CommaSymbol, 0, 1>,
    }
);

impl NodeConfig for StructFieldGroup {}

impl StructFieldGroup {
    fn iter(&self) -> impl Iterator<Item = &StructField> {
        iter::once(&self.first_field)
            .chain(self.other_fields.iter().map(|other| &other.field))
            .map(|field| &**field)
    }

    fn field(&self, name: &str) -> Option<&StructField> {
        self.iter().find(|field| field.ident.slice == name)
    }
}

sequence!(
    struct StructField {
        pub_: Repeated<PubKeyword, 0, 1>,
        ident: Ident,
        #[force_error(true)]
        colon: ColonSymbol,
        type_: Type,
    }
);

impl NodeConfig for StructField {
    fn is_public(&self) -> bool {
        self.pub_.iter().len() > 0
    }

    fn is_ref(&self, _index: &NodeIndex) -> Option<bool> {
        Some(true)
    }

    fn type_<'a>(&'a self, index: &'a NodeIndex) -> Option<NodeType<'a>> {
        self.type_.type_(index)
    }

    fn is_transpilable_dependency(&self, _index: &NodeIndex) -> bool {
        false
    }

    fn transpile(&self, ctx: &mut TranspilationContext<'_>) -> String {
        let id = self.id;
        let type_ = self.type_.transpile(ctx);
        format!("_{id}: {type_},")
    }
}

impl StructField {
    fn type_item<'a>(&self, index: &'a NodeIndex) -> &'a dyn TypeItem {
        self.type_
            .item(index)
            .expect("internal error: invalid field type")
    }
}

sequence!(
    #[allow(unused_mut)]
    struct StructOtherField {
        commas: CommaSymbol,
        field: StructField,
    }
);

impl NodeConfig for StructOtherField {}

pub(crate) fn to_item(node: &dyn Node) -> &dyn TypeItem {
    if let Some(type_) = (node as &dyn Any).downcast_ref::<NativeStructItem>() {
        type_
    } else if let Some(type_) = (node as &dyn Any).downcast_ref::<StructItem>() {
        type_
    } else {
        unreachable!("unknown type item")
    }
}

fn field_offset(fields: &[&StructField], index: &NodeIndex) -> u32 {
    if fields.len() == 1 {
        0
    } else {
        let last_field_type = fields[fields.len() - 1].type_item(index);
        let before_last_field_type = fields[fields.len() - 2].type_item(index);
        let last_field_alignment = last_field_type.alignment(index);
        let before_last_field_size = before_last_field_type.size(index);
        round_up(
            last_field_alignment,
            field_offset(&fields[..fields.len() - 1], index) + before_last_field_size,
        )
    }
}

fn round_up(k: u32, n: u32) -> u32 {
    n.div_ceil(k) * k
}
