use crate::compilation::index::NodeIndex;
use crate::compilation::node::{pattern, NodeConfig, NodeType};
use crate::compilation::transpilation::TranspilationContext;
use crate::compilation::validation::ValidationContext;
use crate::language::keywords::RESERVED_KEYWORDS;
use crate::language::sources;
use crate::ValidationError;

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
        let source = index
            .search(
                "`f32` type",
                sources::type_criteria(),
                &self.path,
                &self.parent_ids,
            )
            .expect("internal error: `f32` type not found");
        Some(NodeType::Source(source))
    }

    fn validate(&self, ctx: &mut ValidationContext<'_>) {
        if !self
            .slice
            .replace('_', "")
            .parse::<f32>()
            .is_ok_and(|value| !value.is_infinite())
        {
            ctx.errors.push(ValidationError::error(
                ctx,
                self,
                "out of bound `f32` literal",
                None,
                &[],
            ));
        }
    }

    fn transpile(&self, _ctx: &mut TranspilationContext<'_>) -> String {
        let value = self.slice.replace('_', "");
        format!("f32({value})")
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
        let source = index
            .search(
                "`u32` type",
                sources::type_criteria(),
                &self.path,
                &self.parent_ids,
            )
            .expect("internal error: `u32` type not found");
        Some(NodeType::Source(source))
    }

    fn validate(&self, ctx: &mut ValidationContext<'_>) {
        if self.slice.replace(['_', 'u'], "").parse::<u32>().is_err() {
            ctx.errors.push(ValidationError::error(
                ctx,
                self,
                "out of bound `u32` literal",
                None,
                &[],
            ));
        }
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
        let source = index
            .search(
                "`i32` type",
                sources::type_criteria(),
                &self.path,
                &self.parent_ids,
            )
            .expect("internal error: `i32` type not found");
        Some(NodeType::Source(source))
    }

    fn validate(&self, ctx: &mut ValidationContext<'_>) {
        if self.slice.replace('_', "").parse::<i32>().is_err() {
            ctx.errors.push(ValidationError::error(
                ctx,
                self,
                "out of bound `i32` literal",
                None,
                &[],
            ));
        }
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
