use crate::GenericParam;
use shad_error::{ErrorLevel, LocatedMessage, SemanticError};
use shad_parser::{AstExpr, AstGenerics, AstIdent, AstItemGenerics};

pub(crate) fn duplicated_param(
    duplicated_param: &GenericParam,
    existing_param: &GenericParam,
) -> SemanticError {
    SemanticError::new(
        format!(
            "generic parameter with name `{}` is defined multiple times",
            duplicated_param.name().label
        ),
        vec![
            LocatedMessage {
                level: ErrorLevel::Error,
                span: duplicated_param.name().span.clone(),
                text: "duplicated generic parameter name".into(),
            },
            LocatedMessage {
                level: ErrorLevel::Info,
                span: existing_param.name().span.clone(),
                text: "generic parameter with same name is defined here".into(),
            },
        ],
    )
}

pub(crate) fn invalid_generic_count(
    expected: &AstItemGenerics,
    actual: &AstGenerics,
) -> SemanticError {
    SemanticError::new(
        format!(
            "expected {} generic parameters, got {} parameters",
            expected.params.len(),
            actual.args.len(),
        ),
        vec![
            LocatedMessage {
                level: ErrorLevel::Error,
                span: actual.span.clone(),
                text: "invalid number of generic parameters".into(),
            },
            LocatedMessage {
                level: ErrorLevel::Info,
                span: expected.span.clone(),
                text: "expected generic parameters".into(),
            },
        ],
    )
}

pub(crate) fn invalid_generic_type(
    generic_value: &AstExpr,
    generic_param_name: &AstIdent,
) -> SemanticError {
    SemanticError::new(
        "expected type, got constant expression",
        vec![
            LocatedMessage {
                level: ErrorLevel::Error,
                span: generic_value.span.clone(),
                text: "invalid type".into(),
            },
            LocatedMessage {
                level: ErrorLevel::Info,
                span: generic_param_name.span.clone(),
                text: "type is expected".into(),
            },
        ],
    )
}

pub(crate) fn invalid_generic_constant(
    generic_value: &AstExpr,
    generic_param_name: &AstIdent,
) -> SemanticError {
    SemanticError::new(
        "expected constant expression, got type",
        vec![
            LocatedMessage {
                level: ErrorLevel::Error,
                span: generic_value.span.clone(),
                text: "invalid constant expression".into(),
            },
            LocatedMessage {
                level: ErrorLevel::Info,
                span: generic_param_name.span.clone(),
                text: "constant expression is expected".into(),
            },
        ],
    )
}
