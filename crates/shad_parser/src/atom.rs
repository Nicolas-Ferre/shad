use crate::token::{Lexer, Token, TokenType};
use shad_error::{Span, SyntaxError};

/// A parsed identifier.
///
/// It generally corresponds to a variable or function name.
///
/// An identifier matches the regex `[a-zA-Z_][a-zA-Z0-9_]*`.
///
/// # Examples
///
/// - `my_buffer` will be parsed as an identifier in `buf my_buffer = 0;` Shad code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstIdent {
    /// The span of the identifier.
    pub span: Span,
    /// The identifier as a string.
    pub label: String,
    /// In case the identifier corresponds to a local variable, the unique identifier of the
    /// variable.
    ///
    /// Note that this ID is set and used only for analysis purpose,
    /// the `shad_parser` crate assigns the ID `0` to all identifiers.
    pub var_id: u64,
    /// The identifier kind.
    pub kind: AstIdentKind,
}

impl AstIdent {
    pub(crate) fn parse(lexer: &mut Lexer<'_>) -> Result<Self, SyntaxError> {
        let token = parse_token(lexer, TokenType::Ident)?;
        Ok(Self {
            span: token.span,
            label: token.slice.to_string(),
            var_id: 0,
            kind: AstIdentKind::Other,
        })
    }
}

/// An parsed identifier kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AstIdentKind {
    /// A variable definition.
    VarDef,
    /// A function reference.
    FnRef,
    /// A field reference.
    FieldRef,
    /// Another kind.
    Other,
}

/// A parsed literal.
///
/// The following literals are recognized:
/// - `f32` literal, following regex `[0-9][0-9_]*\\.([0-9][0-9_]*)?`.
/// - `u32` literal, following regex `[0-9][0-9_]*u`.
/// - `i32` literal, following regex `[0-9][0-9_]*`.
///
/// # Examples
///
/// - Shad code `1.` will be parsed as an `f32` literal.
/// - Shad code `1.2` will be parsed as an `f32` literal.
/// - Shad code `1_000.420_456` will be parsed as an `f32` literal.
/// - Shad code `123u` will be parsed as a `u32` literal.
/// - Shad code `123` will be parsed as an `i32` literal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstLiteral {
    /// The span of the literal.
    pub span: Span,
    /// The raw value of the literal.
    pub raw_value: String,
    /// The value of the literal without underscores.
    pub cleaned_value: String,
    /// The type of the literal.
    pub type_: AstLiteralType,
}

impl AstLiteral {
    #[allow(clippy::wildcard_enum_match_arm)]
    pub(crate) fn parse(lexer: &mut Lexer<'_>) -> Result<Self, SyntaxError> {
        let token = lexer.next_token()?;
        Ok(Self {
            span: token.span,
            raw_value: token.slice.to_string(),
            cleaned_value: token.slice.replace('_', ""),
            type_: match token.type_ {
                TokenType::F32Literal => AstLiteralType::F32,
                TokenType::U32Literal => AstLiteralType::U32,
                TokenType::I32Literal => AstLiteralType::I32,
                TokenType::True | TokenType::False => AstLiteralType::Bool,
                _ => unreachable!("internal error: not supported literal"),
            },
        })
    }
}

/// The type of a literal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AstLiteralType {
    /// The `f32` primitive type.
    F32,
    /// The `u32` primitive type.
    U32,
    /// The `i32` primitive type.
    I32,
    /// The `bool` primitive type.
    Bool,
}

pub(crate) fn parse_token<'a>(
    lexer: &mut Lexer<'a>,
    expected_type: TokenType,
) -> Result<Token<'a>, SyntaxError> {
    let token = lexer.next_token()?;
    if token.type_ == expected_type {
        Ok(token)
    } else {
        Err(SyntaxError::new(
            token.span.start,
            lexer.module(),
            format!("expected {}", expected_type.label()),
        ))
    }
}

pub(crate) fn parse_token_option<'a>(
    lexer: &mut Lexer<'a>,
    expected_type: TokenType,
) -> Result<Option<Token<'a>>, SyntaxError> {
    if parse_token(&mut lexer.clone(), expected_type).is_ok() {
        parse_token(lexer, expected_type).map(Some)
    } else {
        Ok(None)
    }
}
