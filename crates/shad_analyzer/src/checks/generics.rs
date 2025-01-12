use crate::registration::generics::{ConstantGenericParam, GenericParam};
use crate::{errors, resolving, Analysis, GenericValue};
use fxhash::FxHashMap;
use shad_error::SemanticError;
use shad_parser::{AstFnCall, AstStatement, Visit};

const SUPPORTED_CONST_TYPES: &[&str] = &["u32", "i32", "f32", "bool"];

pub(crate) fn check(analysis: &mut Analysis) {
    let mut errors = vec![];
    check_params(analysis, &mut errors);
    check_args(analysis, &mut errors);
    analysis.errors.extend(errors);
}

fn check_params(analysis: &Analysis, errors: &mut Vec<SemanticError>) {
    for type_ in analysis.types.values() {
        check_item_params(errors, &type_.generics);
    }
    for fn_ in analysis.raw_fns.values() {
        check_item_params(errors, &fn_.generics);
    }
}

fn check_item_params(errors: &mut Vec<SemanticError>, generics: &[GenericParam]) {
    for param in generics {
        if let GenericParam::Constant(ConstantGenericParam {
            type_name,
            type_id: Some(type_id),
            ..
        }) = param
        {
            if type_id.module.is_some() || !SUPPORTED_CONST_TYPES.contains(&type_id.name.as_str()) {
                let error = errors::constants::unsupported_type(type_name);
                errors.push(error);
            }
        }
    }
    let mut name_params = FxHashMap::default();
    for param in generics {
        if let Some(duplicated_param) = name_params.insert(&param.name().label, param) {
            let error = errors::generics::duplicated_param(duplicated_param, param);
            errors.push(error);
        }
    }
}

fn check_args(analysis: &Analysis, errors: &mut Vec<SemanticError>) {
    for block in &analysis.init_blocks {
        check_statement_args(analysis, errors, &block.ast.statements);
    }
    for block in &analysis.run_blocks {
        check_statement_args(analysis, errors, &block.ast.statements);
    }
    for fn_ in analysis.raw_fns.values() {
        check_statement_args(analysis, errors, &fn_.ast.statements);
    }
}

fn check_statement_args(
    analysis: &Analysis,
    errors: &mut Vec<SemanticError>,
    statements: &[AstStatement],
) {
    for statement in statements {
        ArgCheck::new(analysis, errors).visit_statement(statement);
    }
}

struct ArgCheck<'a> {
    analysis: &'a Analysis,
    errors: &'a mut Vec<SemanticError>,
}

impl<'a> ArgCheck<'a> {
    fn new(analysis: &'a Analysis, errors: &'a mut Vec<SemanticError>) -> Self {
        Self { analysis, errors }
    }
}

impl Visit for ArgCheck<'_> {
    fn enter_fn_call(&mut self, node: &AstFnCall) {
        let (Some(fn_), Some(specialized_fn)) = (
            resolving::items::fn_(self.analysis, node, true),
            resolving::items::fn_(self.analysis, node, false),
        ) else {
            return;
        };
        if let (Some(arg_type_ids), Some(param_type_ids)) = (
            resolving::types::fn_args(self.analysis, node),
            fn_.params
                .iter()
                .map(|param| param.type_id.clone())
                .collect::<Option<Vec<_>>>(),
        ) {
            if arg_type_ids != param_type_ids {
                self.errors.push(errors::functions::not_found(
                    node,
                    &arg_type_ids,
                    &specialized_fn.id.generic_values,
                ));
                return;
            }
        }
        if fn_.ast.generics.params.len() != node.generics.args.len() {
            self.errors.push(errors::generics::invalid_generic_count(
                &fn_.ast.generics,
                &node.generics,
            ));
        }
        for (((param, arg), param_ast), arg_ast) in fn_
            .generics
            .iter()
            .zip(&specialized_fn.id.generic_values)
            .zip(&fn_.ast.generics.params)
            .zip(&node.generics.args)
        {
            match (param, arg) {
                (GenericParam::Constant(_), GenericValue::Type(_)) => {
                    self.errors.push(errors::generics::invalid_generic_constant(
                        arg_ast,
                        &param_ast.name,
                    ));
                }
                (GenericParam::Type(_), GenericValue::Constant(_)) => {
                    self.errors.push(errors::generics::invalid_generic_type(
                        arg_ast,
                        &param_ast.name,
                    ));
                }
                (GenericParam::Constant(param), GenericValue::Constant(arg)) => {
                    if let Some(expected_type) = &param.type_id {
                        let actual_type = arg.type_id();
                        if expected_type != &actual_type {
                            self.errors.push(errors::expressions::invalid_type(
                                &param.type_name.span,
                                &arg_ast.span,
                                expected_type,
                                &actual_type,
                            ));
                        }
                    }
                }
                (GenericParam::Type(_), GenericValue::Type(_)) => (),
            }
        }
    }
}
