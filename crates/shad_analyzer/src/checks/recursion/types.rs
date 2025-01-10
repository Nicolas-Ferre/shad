use crate::checks::recursion::{ItemRecursionCheck, UsedItem};
use crate::{errors, Analysis, Type, TypeId};
use fxhash::FxHashSet;
use shad_parser::AstStructItem;
use std::mem;

pub(crate) fn check(analysis: &mut Analysis) {
    let mut errors = vec![];
    let mut errored_type_ids = FxHashSet::default();
    for (type_id, type_) in &analysis.types {
        if let Some(ast) = &type_.ast {
            let mut checker = ItemRecursionCheck::new(
                analysis,
                type_id.clone(),
                mem::take(&mut errored_type_ids),
            );
            visit(&mut checker, type_, ast);
            errors.extend(checker.errors);
            errored_type_ids = checker.errored_item_ids;
        }
    }
    analysis.errors.extend(errors);
}

fn visit(checker: &mut ItemRecursionCheck<'_, TypeId>, type_: &Type, ast: &AstStructItem) {
    for (ast_field, field) in ast.fields.iter().zip(&type_.fields) {
        let field_type = &checker.analysis.types[field
            .type_id
            .as_ref()
            .expect("internal error: missing field type")];
        if let Some(field_type_ast) = &field_type.ast {
            checker.used_item_ids.push(UsedItem {
                usage_span: ast_field.type_.span.clone(),
                def_span: field_type_ast.name.span.clone(),
                id: field_type.id.clone(),
                name: field_type.name.clone(),
            });
            if !checker.detect_error(|analysis, type_id, type_stack| {
                errors::types::recursion_found(&analysis.types[type_id], type_stack)
            }) {
                visit(checker, field_type, field_type_ast);
            }
            checker.used_item_ids.pop();
        }
    }
}
