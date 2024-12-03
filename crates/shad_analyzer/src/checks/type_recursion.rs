use crate::{errors, Analysis, Type, TypeId};
use fxhash::FxHashSet;
use shad_error::{SemanticError, Span};
use shad_parser::AstStructItem;
use std::mem;

pub(crate) fn check(analysis: &mut Analysis) {
    let mut errors = vec![];
    let mut errored_type_ids = FxHashSet::default();
    for (type_id, type_) in &analysis.types {
        if let Some(ast) = &type_.ast {
            let mut checker = StructRecursionCheck::new(
                analysis,
                type_id.clone(),
                mem::take(&mut errored_type_ids),
            );
            checker.visit(type_, ast);
            errors.extend(checker.errors);
            errored_type_ids = checker.errored_type_ids;
        }
    }
    analysis.errors.extend(errors);
}

#[derive(Debug)]
pub(crate) struct UsedType {
    pub(crate) usage_span: Span,
    pub(crate) def_span: Span,
    pub(crate) id: TypeId,
}

struct StructRecursionCheck<'a> {
    analysis: &'a Analysis,
    current_type_id: TypeId,
    used_type_ids: Vec<UsedType>,
    errored_type_ids: FxHashSet<TypeId>,
    errors: Vec<SemanticError>,
}

impl<'a> StructRecursionCheck<'a> {
    fn new(analysis: &'a Analysis, type_id: TypeId, errored_type_ids: FxHashSet<TypeId>) -> Self {
        Self {
            analysis,
            current_type_id: type_id,
            used_type_ids: vec![],
            errored_type_ids,
            errors: vec![],
        }
    }

    fn detect_error(&mut self) -> bool {
        if !self.is_last_usage_recursive() {
            false
        } else if self.is_error_already_generated() {
            true
        } else {
            for call in &self.used_type_ids {
                self.errored_type_ids.insert(call.id.clone());
            }
            self.errored_type_ids.insert(self.current_type_id.clone());
            self.errors.push(errors::types::recursion_found(
                &self.current_type_id,
                &self.used_type_ids,
            ));
            true
        }
    }

    fn is_last_usage_recursive(&self) -> bool {
        self.used_type_ids
            .last()
            .map_or(false, |last_call| last_call.id == self.current_type_id)
    }

    fn is_error_already_generated(&self) -> bool {
        for call in &self.used_type_ids {
            if self.errored_type_ids.contains(&call.id) {
                return true;
            }
        }
        self.errored_type_ids.contains(&self.current_type_id)
    }

    fn visit(&mut self, type_: &Type, ast: &AstStructItem) {
        for (ast_field, field) in ast.fields.iter().zip(&type_.fields) {
            let field_type = &self.analysis.types[field
                .type_id
                .as_ref()
                .expect("internal error: missing field type")];
            if let Some(field_type_ast) = &field_type.ast {
                self.used_type_ids.push(UsedType {
                    usage_span: ast_field.type_.span.clone(),
                    def_span: field_type_ast.name.span.clone(),
                    id: field_type.id.clone(),
                });
                if !self.detect_error() {
                    self.visit(field_type, field_type_ast);
                }
                self.used_type_ids.pop();
            }
        }
    }
}
