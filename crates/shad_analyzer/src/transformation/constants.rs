use crate::registration::constants::ConstantValue;
use crate::{Analysis, IdentSource};
use shad_parser::{AstExpr, AstExprRoot, AstLiteral, AstLiteralType, AstStatement, VisitMut};
use std::mem;

pub(crate) fn transform(analysis: &mut Analysis) {
    super::transform_statements(analysis, |analysis, statements| {
        *statements = mem::take(statements)
            .into_iter()
            .flat_map(|mut statement| {
                let mut transform = ConstantTransform::new(analysis);
                transform.visit_statement(&mut statement);
                transform.statements.push(statement);
                transform.statements
            })
            .collect();
    });
}

struct ConstantTransform<'a> {
    analysis: &'a mut Analysis,
    statements: Vec<AstStatement>,
}

impl<'a> ConstantTransform<'a> {
    fn new(analysis: &'a mut Analysis) -> Self {
        Self {
            analysis,
            statements: vec![],
        }
    }
}

impl VisitMut for ConstantTransform<'_> {
    fn enter_expr(&mut self, node: &mut AstExpr) {
        if let AstExprRoot::Ident(ident) = &node.root {
            if let IdentSource::Constant(constant_id) = &self.analysis.idents[&ident.id].source {
                let value = self.analysis.constants[constant_id]
                    .value
                    .clone()
                    .expect("internal error: not calculated constant");
                node.root = AstExprRoot::Literal(match value {
                    ConstantValue::U32(value) => AstLiteral {
                        span: node.root.span().clone(),
                        value: format!("{value}u"),
                        type_: AstLiteralType::U32,
                        is_neg: false,
                    },
                    ConstantValue::I32(value) => AstLiteral {
                        span: node.root.span().clone(),
                        value: value.to_string(),
                        type_: AstLiteralType::I32,
                        is_neg: false,
                    },
                    ConstantValue::F32(value) => AstLiteral {
                        span: node.root.span().clone(),
                        value: {
                            let value = value.to_string();
                            if value.contains('.') {
                                value
                            } else {
                                format!("{value}.0")
                            }
                        },
                        type_: AstLiteralType::F32,
                        is_neg: false,
                    },
                    ConstantValue::Bool(value) => AstLiteral {
                        span: node.root.span().clone(),
                        value: value.to_string(),
                        type_: AstLiteralType::Bool,
                        is_neg: false,
                    },
                });
            }
        }
    }
}
