use crate::compilation::index::NodeIndex;
use crate::compilation::node::{
    sequence, Node, NodeConfig, NodeSourceSearchCriteria, NodeType, Repeated,
};
use crate::compilation::transpilation::TranspilationContext;
use crate::compilation::validation::ValidationContext;
use crate::language::keywords::{
    CloseCurlyBracketSymbol, ColonSymbol, CommaSymbol, EqSymbol, NativeKeyword,
    OpenCurlyBracketSymbol, StructKeyword,
};
use crate::language::patterns::{Ident, StringLiteral, U32Literal};
use crate::language::{items, sources};
use std::any::Any;
use std::iter;

pub(crate) const NO_RETURN_TYPE: &str = "<no return>";

sequence!(
    struct NativeStructItem {
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

    fn validate(&self, ctx: &mut ValidationContext<'_>) {
        items::check_duplicated_items(self, ctx);
    }

    fn is_transpilable_dependency(&self, _index: &NodeIndex) -> bool {
        false
    }
}

impl NativeStructItem {
    pub(crate) fn field(&self, name: &str) -> Option<&StructField> {
        self.fields.iter().find_map(|field| field.field(name))
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
    fn field(&self, name: &str) -> Option<&StructField> {
        iter::once(&self.first_field)
            .chain(self.other_fields.iter().map(|other| &other.field))
            .find(|field| field.ident.slice == name)
            .map(|field| &**field)
    }
}

sequence!(
    struct StructField {
        ident: Ident,
        #[force_error(true)]
        colon: ColonSymbol,
        type_: Type,
    }
);

impl NodeConfig for StructField {
    fn type_<'a>(&self, index: &'a NodeIndex) -> Option<NodeType<'a>> {
        self.type_.type_(index)
    }

    fn is_transpilable_dependency(&self, _index: &NodeIndex) -> bool {
        false
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
        sources::check_missing_source(self, ctx);
    }

    fn transpile(&self, ctx: &mut TranspilationContext<'_>) -> String {
        transpile_name(
            self.source(ctx.index)
                .expect("internal error: type not found"),
        )
    }
}

pub(crate) fn size(type_: &dyn Node) -> u64 {
    if let Some(type_) = (type_ as &dyn Any).downcast_ref::<NativeStructItem>() {
        type_
            .size
            .slice
            .replace(['_', 'u'], "")
            .parse::<u64>()
            .expect("internal error: invalid type size")
    } else {
        unreachable!("unknown type size")
    }
}

pub(crate) fn name(type_: &dyn Node) -> String {
    if let Some(type_) = (type_ as &dyn Any).downcast_ref::<NativeStructItem>() {
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

pub(crate) fn transpile_name(type_: &dyn Node) -> String {
    if let Some(type_) = (type_ as &dyn Any).downcast_ref::<NativeStructItem>() {
        type_.transpilation.as_str().to_string()
    } else {
        unreachable!("unknown type item")
    }
}
