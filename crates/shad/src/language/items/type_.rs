use crate::compilation::index::NodeIndex;
use crate::compilation::node::{
    sequence, Node, NodeConfig, NodeSourceSearchCriteria, NodeType, Repeated,
};
use crate::compilation::transpilation::TranspilationContext;
use crate::compilation::validation::ValidationContext;
use crate::language::keywords::{
    CloseCurlyBracketSymbol, ColonSymbol, CommaSymbol, EqSymbol, NativeKeyword,
    OpenCurlyBracketSymbol, PubKeyword, StructKeyword,
};
use crate::language::patterns::{Ident, StringLiteral, U32Literal};
use crate::language::{sources, validations};
use crate::ValidationError;
use indoc::indoc;
use itertools::Itertools;
use std::any::Any;
use std::iter;

pub(crate) const NO_RETURN_TYPE: &str = "<no return>";

sequence!(
    struct NativeStructItem {
        pub_: Repeated<PubKeyword, 0, 1>,
        native: NativeKeyword,
        struct_: StructKeyword,
        #[force_error(true)]
        ident: Ident,
        eq: EqSymbol,
        transpilation: StringLiteral,
        comma1: CommaSymbol,
        alignment: U32Literal,
        comma2: CommaSymbol,
        size: U32Literal,
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
        validations::check_duplicated_items(self, ctx);
    }

    fn is_transpilable_dependency(&self, _index: &NodeIndex) -> bool {
        false
    }
}

impl NativeStructItem {}

sequence!(
    struct StructItem {
        pub_: Repeated<PubKeyword, 0, 1>,
        struct_: StructKeyword,
        #[force_error(true)]
        ident: Ident,
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
    fn is_ref(&self, _index: &NodeIndex) -> Option<bool> {
        Some(true)
    }

    fn is_public(&self) -> bool {
        self.pub_.iter().len() > 0
    }

    fn type_<'a>(&self, index: &'a NodeIndex) -> Option<NodeType<'a>> {
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
    fn type_source<'a>(&self, index: &'a NodeIndex) -> &'a dyn Node {
        self.type_
            .source(index)
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

sequence!(
    #[allow(unused_mut)]
    struct Type {
        ident: Ident,
    }
);

impl NodeConfig for Type {
    fn source_key(&self, _index: &NodeIndex) -> Option<String> {
        Some(sources::type_key(&self.ident))
    }

    fn source<'a>(&self, index: &'a NodeIndex) -> Option<&'a dyn Node> {
        index.search(self, &self.source_key(index)?)
    }

    fn source_search_criteria(&self) -> &'static [NodeSourceSearchCriteria] {
        sources::type_criteria()
    }

    fn type_<'a>(&self, index: &'a NodeIndex) -> Option<NodeType<'a>> {
        self.source(index).map(NodeType::Source)
    }

    fn validate(&self, ctx: &mut ValidationContext<'_>) {
        validations::check_missing_source(self, ctx);
    }

    fn transpile(&self, ctx: &mut TranspilationContext<'_>) -> String {
        transpile_name(
            self.source(ctx.index)
                .expect("internal error: type not found"),
        )
    }
}

pub(crate) fn is_native(type_: &dyn Node) -> bool {
    (type_ as &dyn Any)
        .downcast_ref::<NativeStructItem>()
        .is_some()
}

pub(crate) fn size(type_: &dyn Node, index: &NodeIndex) -> u64 {
    if let Some(type_) = (type_ as &dyn Any).downcast_ref::<NativeStructItem>() {
        type_
            .size
            .value()
            .expect("internal error: invalid u32 literal for struct size")
            .into()
    } else if let Some(type_) = (type_ as &dyn Any).downcast_ref::<StructItem>() {
        let fields: Vec<_> = type_.fields.iter().collect();
        let last_field_size = size(fields[fields.len() - 1].type_source(index), index);
        round_up(
            alignment(type_, index),
            field_offset(&fields, index) + last_field_size,
        )
    } else {
        unreachable!("unknown type size")
    }
}

fn alignment(type_: &dyn Node, index: &NodeIndex) -> u64 {
    if let Some(type_) = (type_ as &dyn Any).downcast_ref::<NativeStructItem>() {
        type_
            .alignment
            .value()
            .expect("internal error: invalid u32 literal for struct alignment")
            .into()
    } else if let Some(type_) = (type_ as &dyn Any).downcast_ref::<StructItem>() {
        type_
            .fields
            .iter()
            .map(|field| alignment(field.type_source(index), index))
            .max()
            .expect("internal error: custom structs should have at least one field")
    } else {
        unreachable!("unknown type size")
    }
}

fn field_offset(fields: &[&StructField], index: &NodeIndex) -> u64 {
    if fields.len() == 1 {
        0
    } else {
        let last_field_type = fields[fields.len() - 1].type_source(index);
        let before_last_field_type = fields[fields.len() - 2].type_source(index);
        let last_field_alignment = alignment(last_field_type, index);
        let before_last_field_size = size(before_last_field_type, index);
        round_up(
            last_field_alignment,
            field_offset(&fields[..fields.len() - 1], index) + before_last_field_size,
        )
    }
}

pub(crate) fn name(type_: &dyn Node) -> String {
    if let Some(type_) = (type_ as &dyn Any).downcast_ref::<NativeStructItem>() {
        type_.ident.slice.clone()
    } else if let Some(type_) = (type_ as &dyn Any).downcast_ref::<StructItem>() {
        type_.ident.slice.clone()
    } else {
        unreachable!("unknown type item")
    }
}

pub(crate) fn name_or_no_return(type_: NodeType<'_>) -> String {
    match type_ {
        NodeType::Source(source) => name(source),
        NodeType::NoReturn => NO_RETURN_TYPE.into(),
    }
}

pub(crate) fn fields(type_: &dyn Node) -> Vec<&StructField> {
    if (type_ as &dyn Any)
        .downcast_ref::<NativeStructItem>()
        .is_some()
    {
        unreachable!("never called for native structs")
    } else if let Some(type_) = (type_ as &dyn Any).downcast_ref::<StructItem>() {
        type_.fields.iter().collect()
    } else {
        unreachable!("unknown type item")
    }
}

pub(crate) fn field<'a>(type_: &'a dyn Node, field_name: &str) -> Option<&'a StructField> {
    if let Some(type_) = (type_ as &dyn Any).downcast_ref::<NativeStructItem>() {
        Some(
            type_
                .fields
                .iter()
                .find_map(|fields| fields.field(field_name))?,
        )
    } else if let Some(type_) = (type_ as &dyn Any).downcast_ref::<StructItem>() {
        type_.fields.field(field_name)
    } else {
        unreachable!("unknown type item")
    }
}

pub(crate) fn transpile_name(type_: &dyn Node) -> String {
    if let Some(type_) = (type_ as &dyn Any).downcast_ref::<NativeStructItem>() {
        type_.transpilation.as_str().to_string()
    } else if let Some(type_) = (type_ as &dyn Any).downcast_ref::<StructItem>() {
        format!("_{}", type_.id)
    } else {
        unreachable!("unknown type item")
    }
}

pub(crate) fn transpile_field_name(type_: &dyn Node, field_name: &str) -> String {
    if (type_ as &dyn Any)
        .downcast_ref::<NativeStructItem>()
        .is_some()
    {
        field_name.into()
    } else if let Some(type_) = (type_ as &dyn Any).downcast_ref::<StructItem>() {
        let field = field(type_, field_name).expect("internal error: field not found");
        format!("_{}", field.id)
    } else {
        unreachable!("unknown type item")
    }
}

fn round_up(k: u64, n: u64) -> u64 {
    n.div_ceil(k) * k
}
