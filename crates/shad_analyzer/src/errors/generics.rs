use crate::GenericParam;
use shad_error::{ErrorLevel, LocatedMessage, SemanticError};

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
