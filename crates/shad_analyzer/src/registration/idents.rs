use crate::registration::types;
use crate::{errors, Analysis, Buffer, BufferId, FnId, Function};
use fxhash::FxHashMap;
use shad_parser::{
    AstBufferItem, AstFnCall, AstFnItem, AstFnQualifier, AstIdent, AstIdentType, AstItem,
    AstVarDefinition, Visit,
};
use std::mem;

/// An analyzed identifier.
#[derive(Debug, Clone)]
pub struct Ident {
    /// The source of the identifier.
    pub source: IdentSource,
    /// The type of the identifier.
    pub type_: Option<String>,
}

impl Ident {
    pub(crate) fn new(source: IdentSource, type_: Option<String>) -> Self {
        Self { source, type_ }
    }
}

/// The source of an identifier.
#[derive(Debug, Clone)]
pub enum IdentSource {
    /// A buffer.
    Buffer(BufferId),
    /// A variable.
    Var(u64),
    /// A function.
    Fn(FnId),
}

pub(crate) fn register(analysis: &mut Analysis) {
    register_buffer_init(analysis);
    register_buffer_types(analysis);
    register_run_blocks(analysis);
    register_fns(analysis);
}

fn register_buffer_init(analysis: &mut Analysis) {
    let asts = mem::take(&mut analysis.asts);
    for (module, ast) in &asts {
        for item in &ast.items {
            if let AstItem::Buffer(buffer) = item {
                IdentRegistration::new(analysis, module, Scope::BufDef, true)
                    .visit_buffer_item(buffer);
            }
        }
    }
    analysis.asts = asts;
}

fn register_buffer_types(analysis: &mut Analysis) {
    let buffer_count = count_buffers(analysis);
    let mut typed_buffer_count = 0;
    let mut last_typed_buffer_count = 0;
    let buffers = analysis.buffers.clone();
    while typed_buffer_count < buffer_count {
        for buffer in buffers.values() {
            if analysis.idents[&buffer.ast.name.id].type_.is_none() {
                let module = &buffer.ast.name.span.module.name;
                IdentRegistration::new(analysis, module, Scope::BufDef, false)
                    .visit_buffer_item(&buffer.ast);
            }
        }
        typed_buffer_count = count_typed_buffers(analysis);
        if typed_buffer_count == last_typed_buffer_count {
            break; // recursive buffer init
        }
        last_typed_buffer_count = typed_buffer_count;
    }
}

fn register_run_blocks(analysis: &mut Analysis) {
    let blocks = mem::take(&mut analysis.run_blocks);
    for block in &blocks {
        IdentRegistration::new(analysis, &block.module, Scope::RunBlock, true)
            .visit_run_item(&block.ast);
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
        IdentRegistration::new(analysis, &fn_.ast.name.span.module.name, scope, true)
            .visit_fn_item(&fn_.ast);
    }
}

fn count_buffers(analysis: &Analysis) -> usize {
    analysis
        .idents
        .values()
        .filter(|e| matches!(e.source, IdentSource::Buffer(_)))
        .count()
}

fn count_typed_buffers(analysis: &Analysis) -> usize {
    analysis
        .idents
        .values()
        .filter(|e| matches!(e.source, IdentSource::Buffer(_)))
        .filter(|e| e.type_.is_some())
        .count()
}

struct IdentRegistration<'a> {
    analysis: &'a mut Analysis,
    module: &'a str,
    scope: Scope,
    are_errors_enabled: bool,
    variables: FxHashMap<String, u64>,
}

impl<'a> IdentRegistration<'a> {
    pub(crate) fn new(
        analysis: &'a mut Analysis,
        module: &'a str,
        scope: Scope,
        are_errors_enabled: bool,
    ) -> Self {
        Self {
            analysis,
            module,
            scope,
            are_errors_enabled,
            variables: FxHashMap::default(),
        }
    }

    fn add_variable(&mut self, ident: &AstIdent) {
        self.variables.insert(ident.label.clone(), ident.id);
    }

    fn find_fn(&self, signature: &str) -> Option<(FnId, &Function)> {
        self.analysis
            .visible_modules
            .get(self.module)
            .into_iter()
            .flatten()
            .filter_map(|module| {
                let id = FnId {
                    module: module.clone(),
                    signature: signature.into(),
                };
                self.analysis.fns.get(&id).map(|fn_| (id, fn_))
            })
            .find(|(fn_id, fn_)| fn_.ast.is_pub || fn_id.module == self.module)
    }

    fn find_buffer(&self, name: &str) -> Option<(BufferId, &Buffer)> {
        self.analysis
            .visible_modules
            .get(self.module)
            .into_iter()
            .flatten()
            .filter_map(|module| {
                let id = BufferId {
                    module: module.clone(),
                    name: name.into(),
                };
                self.analysis.buffers.get(&id).map(|buffer| (id, buffer))
            })
            .find(|(buffer_id, buffer)| buffer.ast.is_pub || buffer_id.module == self.module)
    }
}

impl Visit for IdentRegistration<'_> {
    fn enter_fn_item(&mut self, node: &AstFnItem) {
        let return_type = node
            .return_type
            .as_ref()
            .and_then(|type_| types::name(self.analysis, &type_.name));
        let fn_ident = Ident::new(IdentSource::Fn(FnId::new(node)), return_type);
        self.analysis.idents.insert(node.name.id, fn_ident);
        for param in &node.params {
            let param_type = types::name(self.analysis, &param.type_);
            let param_ident = Ident::new(IdentSource::Var(param.name.id), param_type);
            self.analysis.idents.insert(param.name.id, param_ident);
            self.add_variable(&param.name);
        }
    }

    fn exit_buffer_item(&mut self, node: &AstBufferItem) {
        let buffer_type = self.analysis.expr_type(&node.value);
        let buffer_ident = Ident::new(
            IdentSource::Buffer(BufferId {
                module: self.module.into(),
                name: node.name.label.clone(),
            }),
            buffer_type,
        );
        self.analysis.idents.insert(node.name.id, buffer_ident);
    }

    fn exit_var_definition(&mut self, node: &AstVarDefinition) {
        let var_type = self.analysis.expr_type(&node.expr);
        let var_ident = Ident::new(IdentSource::Var(node.name.id), var_type);
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
            if let Some((fn_id, fn_)) = self.find_fn(&signature) {
                if let Some(return_type) = fn_.ast.return_type.clone() {
                    if node.is_statement && self.are_errors_enabled {
                        let error = errors::fn_calls::unexpected_return_type(node, &fn_id);
                        self.analysis.errors.push(error);
                    } else if !node.is_statement {
                        let fn_type = types::name(self.analysis, &return_type.name);
                        let fn_ident = Ident::new(IdentSource::Fn(fn_id), fn_type);
                        self.analysis.idents.insert(node.name.id, fn_ident);
                    }
                } else if node.is_statement {
                    let fn_ident = Ident::new(IdentSource::Fn(fn_id), None);
                    self.analysis.idents.insert(node.name.id, fn_ident);
                } else if self.are_errors_enabled {
                    self.analysis
                        .errors
                        .push(errors::fn_calls::no_return_type(&fn_id, node));
                }
            } else if self.are_errors_enabled {
                self.analysis
                    .errors
                    .push(errors::functions::not_found(node, &signature));
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
            let var_ident = Ident::new(IdentSource::Var(id), var_type);
            self.analysis.idents.insert(node.id, var_ident);
        } else if let (true, Some((buffer_id, buffer))) = (
            self.scope.are_buffers_accessible(),
            self.find_buffer(&node.label),
        ) {
            let buffer_type = self
                .analysis
                .idents
                .get(&buffer.ast.name.id)
                .and_then(|ident| ident.type_.clone());
            let buffer_ident = Ident::new(IdentSource::Buffer(buffer_id), buffer_type);
            self.analysis.idents.insert(node.id, buffer_ident);
        } else if self.are_errors_enabled {
            self.analysis
                .errors
                .push(errors::variables::not_found(node));
        }
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
