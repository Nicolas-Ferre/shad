use crate::registration::functions::signature;
use crate::registration::types;
use crate::{errors, Analysis};
use fxhash::FxHashMap;
use shad_parser::{
    AstBufferItem, AstFnCall, AstFnItem, AstFnQualifier, AstIdent, AstIdentType, AstItem,
    AstVarDefinition, Visit,
};
use std::mem;

#[derive(Debug, Clone)]
pub struct Ident {
    pub source: IdentSource,
    pub type_: Option<String>,
}

impl Ident {
    pub(crate) fn new(source: IdentSource, type_: Option<String>) -> Self {
        Self { source, type_ }
    }
}

#[derive(Debug, Clone)]
pub enum IdentSource {
    Buffer(String),
    Ident(u64),
    Fn(String),
}

pub(crate) fn register(analysis: &mut Analysis) {
    register_buffers(analysis);
    register_run_blocks(analysis);
    register_fns(analysis);
}

fn register_buffers(analysis: &mut Analysis) {
    let items = mem::take(&mut analysis.ast.items);
    for item in &items {
        if let AstItem::Buffer(buffer) = item {
            IdentRegistration::new(analysis, Scope::BufDef).visit_buffer_item(buffer);
        }
    }
    analysis.ast.items = items;
}

fn register_run_blocks(analysis: &mut Analysis) {
    let blocks = mem::take(&mut analysis.run_blocks);
    for block in &blocks {
        IdentRegistration::new(analysis, Scope::RunBlock).visit_run_item(&block.ast);
    }
    analysis.run_blocks = blocks;
}

fn register_fns(analysis: &mut Analysis) {
    for fn_ in analysis.fns.clone().into_values() {
        let scope = if fn_.ast.qualifier == AstFnQualifier::Buf {
            Scope::BufFnBody
        } else {
            Scope::FnBody
        };
        IdentRegistration::new(analysis, scope).visit_fn_item(&fn_.ast);
    }
}

struct IdentRegistration<'a> {
    analysis: &'a mut Analysis,
    scope: Scope,
    variables: FxHashMap<String, u64>,
}

impl<'a> IdentRegistration<'a> {
    pub(crate) fn new(analysis: &'a mut Analysis, scope: Scope) -> Self {
        Self {
            analysis,
            scope,
            variables: FxHashMap::default(),
        }
    }

    fn add_variable(&mut self, ident: &AstIdent) {
        self.variables.insert(ident.label.clone(), ident.id);
    }
}

impl Visit for IdentRegistration<'_> {
    fn enter_fn_item(&mut self, node: &AstFnItem) {
        let return_type = node
            .return_type
            .as_ref()
            .and_then(|type_| types::name(self.analysis, &type_.name));
        let fn_ident = Ident::new(IdentSource::Fn(signature(node)), return_type);
        self.analysis.idents.insert(node.name.id, fn_ident);
        for param in &node.params {
            let param_type = types::name(self.analysis, &param.type_);
            let param_ident = Ident::new(IdentSource::Ident(param.name.id), param_type);
            self.analysis.idents.insert(param.name.id, param_ident);
            self.add_variable(&param.name);
        }
    }

    fn exit_buffer_item(&mut self, node: &AstBufferItem) {
        let buffer_type = self.analysis.expr_type(&node.value);
        let buffer_ident = Ident::new(IdentSource::Buffer(node.name.label.clone()), buffer_type);
        self.analysis.idents.insert(node.name.id, buffer_ident);
    }

    fn exit_var_definition(&mut self, node: &AstVarDefinition) {
        let var_type = self.analysis.expr_type(&node.expr);
        let var_ident = Ident::new(IdentSource::Ident(node.name.id), var_type);
        self.analysis.idents.insert(node.name.id, var_ident);
        self.add_variable(&node.name);
    }

    fn exit_fn_call(&mut self, node: &AstFnCall) {
        if let Some(arg_types) = node
            .args
            .iter()
            .map(|node| self.analysis.expr_type(node))
            .collect::<Option<Vec<_>>>()
        {
            let signature = format!("{}({})", node.name.label, arg_types.join(", "));
            if let Some(fn_) = self.analysis.fns.get(&signature) {
                if let Some(return_type) = fn_.ast.return_type.clone() {
                    if node.is_statement {
                        let error = errors::fn_calls::unexpected_return_type(
                            self.analysis,
                            node,
                            &signature,
                        );
                        self.analysis.errors.push(error);
                    } else {
                        let fn_type = types::name(self.analysis, &return_type.name);
                        let fn_ident = Ident::new(IdentSource::Fn(signature), fn_type);
                        self.analysis.idents.insert(node.name.id, fn_ident);
                    }
                } else if node.is_statement {
                    let fn_ident = Ident::new(IdentSource::Fn(signature), None);
                    self.analysis.idents.insert(node.name.id, fn_ident);
                } else {
                    self.analysis.errors.push(errors::fn_calls::no_return_type(
                        self.analysis,
                        &signature,
                        node,
                    ));
                }
            } else {
                self.analysis.errors.push(errors::functions::not_found(
                    self.analysis,
                    node,
                    &signature,
                ));
            }
        }
    }

    fn exit_ident(&mut self, node: &AstIdent) {
        if node.type_ != AstIdentType::VarUsage {
            return;
        }
        if let Some(&id) = self.variables.get(&node.label) {
            let var_type = self
                .analysis
                .idents
                .get(&id)
                .and_then(|var| var.type_.clone());
            let var_ident = Ident::new(IdentSource::Ident(id), var_type);
            self.analysis.idents.insert(node.id, var_ident);
            return;
        } else if let (true, Some(buffer)) = (
            self.scope.are_buffers_accessible(),
            self.analysis.buffers.get(&node.label),
        ) {
            if let Some(buffer_ident) = self.analysis.idents.get(&buffer.ast.name.id) {
                let buffer_name = buffer.ast.name.label.clone();
                let buffer_type = buffer_ident.type_.clone();
                let buffer_ident = Ident::new(IdentSource::Buffer(buffer_name), buffer_type);
                self.analysis.idents.insert(node.id, buffer_ident);
                return;
            }
        }
        self.analysis
            .errors
            .push(errors::variables::not_found(self.analysis, node));
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Scope {
    BufDef,
    BufFnBody,
    FnBody,
    RunBlock,
}

impl Scope {
    fn are_buffers_accessible(self) -> bool {
        match self {
            Self::BufDef | Self::BufFnBody | Self::RunBlock => true,
            Self::FnBody => false,
        }
    }
}
