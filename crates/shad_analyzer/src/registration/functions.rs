use crate::{errors, resolver, Analysis, Type, TypeId};
use itertools::Itertools;
use shad_parser::{
    AstFnItem, AstFnParam, AstFnQualifier, AstIdent, AstIdentType, AstItem, AstReturnType,
    AstStructItem,
};
use std::mem;

/// An analyzed function.
#[derive(Debug, Clone)]
pub struct Function {
    /// The function AST.
    pub ast: AstFnItem,
    /// The unique ID of the function.
    pub id: FnId,
    /// Whether the function will be inlined.
    pub is_inlined: bool,
    /// The return type ID.
    pub return_type_id: Option<TypeId>,
    /// The analyzed function parameters.
    pub params: Vec<FnParam>,
    /// The type from which the function has been generated.
    pub source_type: Option<TypeId>,
}

impl Function {
    /// Whether the function has a `ref` parameter.
    pub fn is_inlined(&self) -> bool {
        is_inlined(&self.ast)
    }
}

/// An analyzed function parameter.
#[derive(Debug, Clone)]
pub struct FnParam {
    /// The parameter name.
    pub name: AstIdent,
    /// The parameter type ID.
    pub type_id: Option<TypeId>,
}

/// The unique identifier of a function.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FnId {
    /// The module in which the function is defined.
    pub module: String,
    /// The function name.
    pub name: String,
    /// The function parameter types.
    pub param_types: Vec<Option<TypeId>>,
}

impl FnId {
    pub(crate) fn from_item(analysis: &Analysis, fn_: &AstFnItem) -> Self {
        Self {
            module: fn_.name.span.module.name.clone(),
            name: fn_.name.label.clone(),
            param_types: fn_
                .params
                .iter()
                .map(|param| resolver::type_(analysis, &param.type_).ok())
                .collect(),
        }
    }

    pub(crate) fn initializer(type_: &Type, ast: &AstStructItem) -> Self {
        Self {
            module: ast.name.span.module.name.clone(),
            name: ast.name.label.clone(),
            param_types: type_
                .fields
                .iter()
                .map(|field| field.type_id.clone())
                .collect(),
        }
    }

    pub(crate) fn signature(&self) -> String {
        format!(
            "{}({})",
            self.name,
            self.param_types
                .iter()
                .map(|type_| type_.as_ref().map_or("?", |t| t.name.as_str()))
                .join(", ")
        )
    }
}

pub(crate) fn register(analysis: &mut Analysis) {
    register_initializers(analysis);
    register_ast(analysis);
}

fn register_initializers(analysis: &mut Analysis) {
    for (type_id, type_) in &analysis.types.clone() {
        if let Some(ast) = &type_.ast {
            let id = FnId::initializer(type_, ast);
            let fn_ = struct_initializer_fn(analysis, ast);
            let fn_ = Function {
                ast: fn_.clone(),
                id: id.clone(),
                is_inlined: is_inlined(&fn_),
                return_type_id: Some(type_id.clone()),
                params: type_
                    .fields
                    .iter()
                    .map(|field| FnParam {
                        name: field.name.clone(),
                        type_id: field.type_id.clone(),
                    })
                    .collect(),
                source_type: Some(type_id.clone()),
            };
            analysis.fns.insert(id, fn_);
        }
    }
}

fn register_ast(analysis: &mut Analysis) {
    let asts = mem::take(&mut analysis.asts);
    for ast in asts.values() {
        for items in &ast.items {
            if let AstItem::Fn(fn_ast) = items {
                let id = FnId::from_item(analysis, fn_ast);
                let fn_ = Function {
                    ast: fn_ast.clone(),
                    id: id.clone(),
                    is_inlined: is_inlined(fn_ast),
                    return_type_id: fn_ast.return_type.as_ref().and_then(|return_type| {
                        resolver::type_or_add_error(analysis, &return_type.name)
                    }),
                    params: fn_ast
                        .params
                        .iter()
                        .map(|param| FnParam {
                            name: param.name.clone(),
                            type_id: resolver::type_or_add_error(analysis, &param.type_),
                        })
                        .collect(),
                    source_type: None,
                };
                if let Some(existing_fn) = analysis.fns.insert(id.clone(), fn_) {
                    analysis.errors.push(errors::functions::duplicated(
                        &id,
                        fn_ast,
                        &existing_fn.ast,
                    ));
                }
            }
        }
    }
    analysis.asts = asts;
}

fn struct_initializer_fn(analysis: &mut Analysis, ast: &AstStructItem) -> AstFnItem {
    AstFnItem {
        name: clone_ident(analysis, &ast.name),
        params: ast
            .fields
            .iter()
            .map(|field| AstFnParam {
                name: clone_ident(analysis, &field.name),
                type_: clone_ident(analysis, &field.type_),
                ref_span: None,
            })
            .collect(),
        return_type: Some(AstReturnType {
            name: clone_ident(analysis, &ast.name),
            is_ref: false,
        }),
        qualifier: AstFnQualifier::Gpu,
        statements: vec![],
        is_pub: false,
    }
}

fn clone_ident(analysis: &mut Analysis, ident: &AstIdent) -> AstIdent {
    AstIdent {
        span: ident.span.clone(),
        label: ident.label.clone(),
        id: analysis.next_id(),
        type_: AstIdentType::Other,
    }
}

fn is_inlined(fn_: &AstFnItem) -> bool {
    fn_.qualifier != AstFnQualifier::Gpu && (is_returning_ref(fn_) || has_ref_param(fn_))
}

fn is_returning_ref(fn_: &AstFnItem) -> bool {
    fn_.return_type.as_ref().map_or(false, |type_| type_.is_ref)
}

fn has_ref_param(fn_: &AstFnItem) -> bool {
    fn_.params.iter().any(|param| param.ref_span.is_some())
}
