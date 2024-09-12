use crate::{type_, Asg, AsgBuffer, AsgType};
use shad_error::{ErrorLevel, LocatedMessage, SemanticError, Span};
use shad_parser::{AstExpr, AstIdent, AstLiteral, AstLiteralType};
use std::rc::Rc;
use std::str::FromStr;

const F32_INT_PART_LIMIT: usize = 38;

pub(crate) fn analyze(asg: &mut Asg, expr: &AstExpr) -> AsgExpr {
    match expr {
        AstExpr::Literal(literal) => AsgExpr::Literal({
            let final_value = literal.value.replace('_', "");
            asg.errors.extend(literal_error(asg, literal, &final_value));
            AsgLiteral {
                value: final_value,
                type_: type_::find(asg, literal_type_str(literal)).clone(),
            }
        }),
        AstExpr::Ident(ident) => AsgExpr::Ident({
            if let Some(buffer) = asg.buffers.get(&ident.label) {
                AsgIdent::Buffer(buffer.clone())
            } else {
                asg.errors.push(not_found_ident_error(asg, ident));
                AsgIdent::Invalid
            }
        }),
    }
}

fn literal_type_str(literal: &AstLiteral) -> &str {
    match literal.type_ {
        AstLiteralType::F32 => "f32",
        AstLiteralType::U32 => "u32",
        AstLiteralType::I32 => "i32",
    }
}

fn literal_error(asg: &Asg, literal: &AstLiteral, final_value: &str) -> Option<SemanticError> {
    match literal.type_ {
        AstLiteralType::F32 => f32_literal_error(asg, literal, final_value),
        AstLiteralType::U32 => {
            let digits = &final_value[..final_value.len() - 1];
            int_literal_error::<u32>(asg, literal, digits, "u32")
        }
        AstLiteralType::I32 => int_literal_error::<i32>(asg, literal, final_value, "i32"),
    }
}

fn not_found_ident_error(asg: &Asg, ident: &AstIdent) -> SemanticError {
    SemanticError::new(
        format!("could not find `{}` value", ident.label),
        vec![LocatedMessage {
            level: ErrorLevel::Error,
            span: ident.span,
            text: "undefined identifier".into(),
        }],
        &asg.ast.code,
        &asg.ast.path,
    )
}

fn f32_literal_error(asg: &Asg, literal: &AstLiteral, final_value: &str) -> Option<SemanticError> {
    let digit_count = final_value
        .find('.')
        .expect("internal error: `.` not found in `f32` literal");
    (digit_count > F32_INT_PART_LIMIT).then(|| {
        let span = Span::new(literal.span.start, literal.span.start + digit_count);
        SemanticError::new(
            "`f32` literal with too many digits in integer part",
            vec![
                LocatedMessage {
                    level: ErrorLevel::Error,
                    span,
                    text: format!("found {digit_count} digits"),
                },
                LocatedMessage {
                    level: ErrorLevel::Info,
                    span,
                    text: format!("maximum {F32_INT_PART_LIMIT} digits are expected"),
                },
            ],
            &asg.ast.code,
            &asg.ast.path,
        )
    })
}

fn int_literal_error<T>(
    asg: &Asg,
    literal: &AstLiteral,
    final_value: &str,
    type_name: &str,
) -> Option<SemanticError>
where
    T: FromStr,
{
    let is_literal_invalid = T::from_str(final_value).is_err();
    is_literal_invalid.then(|| {
        SemanticError::new(
            format!("`{type_name}` literal out of range"),
            vec![LocatedMessage {
                level: ErrorLevel::Error,
                span: literal.span,
                text: format!("value is outside allowed range for `{type_name}` type"),
            }],
            &asg.ast.code,
            &asg.ast.path,
        )
    })
}

/// An analyzed expression definition.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum AsgExpr {
    /// A literal.
    Literal(AsgLiteral),
    /// An identifier.
    Ident(AsgIdent),
}

impl AsgExpr {
    /// Returns the type of the expression.
    pub fn type_<'a>(&'a self, asg: &'a Asg) -> &Rc<AsgType> {
        match self {
            Self::Literal(literal) => &literal.type_,
            Self::Ident(ident) => ident.type_(asg),
        }
    }

    pub(crate) fn buffers(&self) -> Vec<Rc<AsgBuffer>> {
        match self {
            Self::Literal(_) => vec![],
            Self::Ident(ident) => ident.buffers(),
        }
    }
}

/// An analyzed literal value.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AsgLiteral {
    /// The literal value.
    pub value: String,
    /// The literal type.
    pub type_: Rc<AsgType>,
}

/// An analyzed identifier.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum AsgIdent {
    /// A buffer identifier.
    Buffer(Rc<AsgBuffer>),
    /// An invalid identifier.
    Invalid,
}

impl AsgIdent {
    fn type_<'a>(&'a self, asg: &'a Asg) -> &Rc<AsgType> {
        match self {
            Self::Buffer(buffer) => buffer.expr.type_(asg),
            Self::Invalid => type_::undefined(asg),
        }
    }

    fn buffers(&self) -> Vec<Rc<AsgBuffer>> {
        match self {
            Self::Buffer(buffer) => vec![buffer.clone()],
            Self::Invalid => vec![],
        }
    }
}
