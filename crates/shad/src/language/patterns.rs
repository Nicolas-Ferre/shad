use crate::compilation::constant::{ConstantContext, ConstantData, ConstantValue};
use crate::compilation::index::NodeIndex;
use crate::compilation::node::{pattern, Node, NodeConfig, NodeType};
use crate::compilation::transpilation::TranspilationContext;
use crate::compilation::validation::ValidationContext;
use crate::compilation::PRELUDE_PATH;
use crate::language::items::type_;
use crate::language::keywords::RESERVED_KEYWORDS;
use crate::language::sources;
use crate::ValidationError;
use std::path::Path;

pattern!(
    Ident,
    "identifier",
    RESERVED_KEYWORDS,
    [
        (1, 1, INCLUDE('a'..='z', 'A'..='Z', '_'..='_')),
        (
            0,
            usize::MAX,
            INCLUDE('a'..='z', 'A'..='Z', '0'..='9', '_'..='_')
        ),
    ],
);

impl NodeConfig for Ident {}

pattern!(
    F32Literal,
    "`f32` literal",
    RESERVED_KEYWORDS,
    [
        (0, 1, INCLUDE('-'..='-')),
        (1, 1, INCLUDE('0'..='9')),
        (0, usize::MAX, INCLUDE('0'..='9', '_'..='_')),
        (1, 1, INCLUDE('.'..='.')),
        (1, 1, INCLUDE('0'..='9')),
        (0, usize::MAX, INCLUDE('0'..='9', '_'..='_')),
    ],
);

impl NodeConfig for F32Literal {
    fn type_<'a>(&self, index: &'a NodeIndex) -> Option<NodeType<'a>> {
        Some(NodeType::Source(self.f32_type(index)))
    }

    fn validate(&self, ctx: &mut ValidationContext<'_>) {
        if self.value().is_none_or(f32::is_infinite) {
            ctx.errors.push(ValidationError::error(
                ctx,
                self,
                "out of bound `f32` literal",
                None,
                &[],
            ));
        }
    }

    fn invalid_constant(&self, _index: &NodeIndex) -> Option<&dyn Node> {
        None
    }

    fn evaluate_constant(&self, ctx: &mut ConstantContext<'_>) -> Option<ConstantValue> {
        Some(ConstantValue {
            transpiled_type_name: type_::transpile_name(self.f32_type(ctx.index)),
            data: ConstantData::F32(self.value()?),
        })
    }

    fn transpile(&self, _ctx: &mut TranspilationContext<'_>) -> String {
        let value = self.slice.replace('_', "");
        format!("f32({value})")
    }
}

impl F32Literal {
    fn value(&self) -> Option<f32> {
        self.slice.replace('_', "").parse::<f32>().ok()
    }

    fn f32_type<'a>(&self, index: &'a NodeIndex) -> &'a dyn Node {
        index
            .search_in_path(
                Path::new(PRELUDE_PATH),
                self,
                "`f32` type",
                sources::type_criteria(),
            )
            .expect("internal error: `f32` type not found")
    }
}

pattern!(
    U32Literal,
    "`u32` literal",
    RESERVED_KEYWORDS,
    [
        (1, 1, INCLUDE('0'..='9')),
        (0, usize::MAX, INCLUDE('0'..='9', '_'..='_')),
        (1, 1, INCLUDE('u'..='u')),
    ],
);

impl NodeConfig for U32Literal {
    fn type_<'a>(&self, index: &'a NodeIndex) -> Option<NodeType<'a>> {
        Some(NodeType::Source(Self::u32_type(self, index)))
    }

    fn validate(&self, ctx: &mut ValidationContext<'_>) {
        if self.value().is_none() {
            ctx.errors.push(ValidationError::error(
                ctx,
                self,
                "out of bound `u32` literal",
                None,
                &[],
            ));
        }
    }

    fn invalid_constant(&self, _index: &NodeIndex) -> Option<&dyn Node> {
        None
    }

    fn evaluate_constant(&self, ctx: &mut ConstantContext<'_>) -> Option<ConstantValue> {
        Some(ConstantValue {
            transpiled_type_name: type_::transpile_name(Self::u32_type(self, ctx.index)),
            data: ConstantData::U32(self.value()?),
        })
    }

    fn transpile(&self, _ctx: &mut TranspilationContext<'_>) -> String {
        let value = self.slice.replace('_', "");
        let value_without_leading_zeros = value.trim_start_matches('0');
        if value_without_leading_zeros.len() == 1 {
            "u32(0u)".into()
        } else {
            format!("u32({value_without_leading_zeros})")
        }
    }
}

impl U32Literal {
    pub(crate) fn u32_type<'a>(node: &impl Node, index: &'a NodeIndex) -> &'a dyn Node {
        index
            .search_in_path(
                Path::new(PRELUDE_PATH),
                node,
                "`u32` type",
                sources::type_criteria(),
            )
            .expect("internal error: `u32` type not found")
    }

    pub(crate) fn value(&self) -> Option<u32> {
        self.slice.replace(['_', 'u'], "").parse::<u32>().ok()
    }
}

pattern!(
    I32Literal,
    "`i32` literal",
    RESERVED_KEYWORDS,
    [
        (0, 1, INCLUDE('-'..='-')),
        (1, 1, INCLUDE('0'..='9')),
        (0, usize::MAX, INCLUDE('0'..='9', '_'..='_')),
    ],
);

impl NodeConfig for I32Literal {
    fn type_<'a>(&self, index: &'a NodeIndex) -> Option<NodeType<'a>> {
        Some(NodeType::Source(Self::i32_type(self, index)))
    }

    fn validate(&self, ctx: &mut ValidationContext<'_>) {
        if self.value().is_none() {
            ctx.errors.push(ValidationError::error(
                ctx,
                self,
                "out of bound `i32` literal",
                None,
                &[],
            ));
        }
    }

    fn invalid_constant(&self, _index: &NodeIndex) -> Option<&dyn Node> {
        None
    }

    fn evaluate_constant(&self, ctx: &mut ConstantContext<'_>) -> Option<ConstantValue> {
        Some(ConstantValue {
            transpiled_type_name: type_::transpile_name(Self::i32_type(self, ctx.index)),
            data: ConstantData::I32(self.value()?),
        })
    }

    fn transpile(&self, _ctx: &mut TranspilationContext<'_>) -> String {
        let value = self.slice.replace('_', "");
        let value_without_leading_zeros = value.trim_start_matches('0');
        if value_without_leading_zeros.is_empty() {
            "i32(0)".into()
        } else {
            format!("i32({value_without_leading_zeros})")
        }
    }
}

impl I32Literal {
    pub(crate) fn i32_type<'a>(node: &impl Node, index: &'a NodeIndex) -> &'a dyn Node {
        index
            .search_in_path(
                Path::new(PRELUDE_PATH),
                node,
                "`i32` type",
                sources::type_criteria(),
            )
            .expect("internal error: `i32` type not found")
    }

    fn value(&self) -> Option<i32> {
        self.slice.replace('_', "").parse::<i32>().ok()
    }
}

pattern!(
    StringLiteral,
    "string literal",
    RESERVED_KEYWORDS,
    [
        (1, 1, INCLUDE('"'..='"')),
        (0, usize::MAX, EXCLUDE('"'..='"')),
        (1, 1, INCLUDE('"'..='"')),
    ],
);

impl NodeConfig for StringLiteral {}

impl StringLiteral {
    pub(crate) fn as_str(&self) -> &str {
        &self.slice[1..self.slice.len() - 1]
    }
}
