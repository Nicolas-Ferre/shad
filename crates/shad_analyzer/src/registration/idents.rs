use crate::{resolving, Analysis, BufferId, FnId, TypeId};
use fxhash::FxHashMap;
use shad_parser::{
    AstBufferItem, AstExpr, AstExprRoot, AstFnCall, AstFnItem, AstFnParam, AstGpuGenericParam,
    AstGpuName, AstIdent, AstItem, AstVarDefinition, Visit,
};
use std::mem;

/// An analyzed identifier.
#[derive(Debug, Clone)]
pub struct Ident {
    /// The source of the identifier.
    pub source: IdentSource,
    /// The type ID of the identifier.
    pub type_id: Option<TypeId>,
}

impl Ident {
    pub(crate) fn new(source: IdentSource, type_id: Option<TypeId>) -> Self {
        Self { source, type_id }
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
    /// A generic type.
    GenericType,
}

pub(crate) fn register(analysis: &mut Analysis) {
    register_structs(analysis);
    register_buffer_init(analysis);
    register_constants(analysis);
    register_buffer_types(analysis);
    register_run_blocks(analysis);
    register_fns(analysis);
}

fn register_structs(analysis: &mut Analysis) {
    for type_ in analysis.types.clone().values() {
        if let Some(name) = &type_
            .ast
            .as_ref()
            .and_then(|ast| ast.gpu_qualifier.as_ref())
            .and_then(|gpu| gpu.name.as_ref())
        {
            register_gpu_name(analysis, name);
        }
    }
}

fn register_constants(analysis: &mut Analysis) {
    for constant in analysis.constants.clone().values() {
        let module = &constant.ast.name.span.module.name;
        IdentRegistration::new(analysis, module, true).visit_const_item(&constant.ast);
    }
}

fn register_buffer_init(analysis: &mut Analysis) {
    let asts = mem::take(&mut analysis.asts);
    for (module, ast) in &asts {
        for item in &ast.items {
            if let AstItem::Buffer(buffer) = item {
                IdentRegistration::new(analysis, module, false).visit_buffer_item(buffer);
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
            if analysis.idents[&buffer.ast.name.id].type_id.is_none() {
                let module = &buffer.ast.name.span.module.name;
                IdentRegistration::new(analysis, module, false).visit_buffer_item(&buffer.ast);
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
        IdentRegistration::new(analysis, &block.module, false).visit_run_item(&block.ast);
    }
    analysis.run_blocks = blocks;
}

fn register_fns(analysis: &mut Analysis) {
    for fn_ in analysis.fns.clone().into_values() {
        IdentRegistration::new(analysis, &fn_.ast.name.span.module.name, false)
            .visit_fn_item(&fn_.ast);
        let name = fn_
            .ast
            .gpu_qualifier
            .as_ref()
            .and_then(|gpu| gpu.name.as_ref());
        if let Some(name) = name {
            register_gpu_name(analysis, name);
        }
    }
}

fn register_gpu_name(analysis: &mut Analysis, name: &AstGpuName) {
    for param in &name.generics {
        if let AstGpuGenericParam::Ident(param) = param {
            let type_id = resolving::items::type_id_or_add_error(analysis, param);
            let ident = Ident::new(IdentSource::GenericType, type_id);
            analysis.idents.insert(param.id, ident);
        }
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
        .filter(|e| e.type_id.is_some())
        .count()
}

struct IdentRegistration<'a> {
    analysis: &'a mut Analysis,
    module: &'a str,
    is_const_context: bool,
    variables: FxHashMap<String, u64>,
}

impl<'a> IdentRegistration<'a> {
    pub(crate) fn new(analysis: &'a mut Analysis, module: &'a str, is_const_context: bool) -> Self {
        Self {
            analysis,
            module,
            is_const_context,
            variables: FxHashMap::default(),
        }
    }

    fn add_variable(&mut self, ident: &AstIdent) {
        self.variables.insert(ident.label.clone(), ident.id);
    }

    fn register_fn_item(&mut self, node: &AstFnItem) {
        let fn_id = FnId::from_item(self.analysis, node);
        let return_type_id = self
            .analysis
            .fns
            .get(&fn_id)
            .and_then(|fn_| fn_.return_type_id.clone());
        let fn_ident_source = IdentSource::Fn(fn_id);
        let fn_ident = Ident::new(fn_ident_source, return_type_id);
        self.analysis.idents.insert(node.name.id, fn_ident);
    }

    fn register_fn_param(&mut self, param: &AstFnParam) {
        let type_id = resolving::items::type_id(self.analysis, &param.type_).ok();
        let ident = Ident::new(IdentSource::Var(param.name.id), type_id);
        self.analysis.idents.insert(param.name.id, ident);
        self.add_variable(&param.name);
    }
}

impl Visit for IdentRegistration<'_> {
    fn enter_fn_item(&mut self, node: &AstFnItem) {
        self.register_fn_item(node);
        for param in &node.params {
            self.register_fn_param(param);
        }
    }

    fn exit_buffer_item(&mut self, node: &AstBufferItem) {
        let buffer_type = resolving::types::expr(self.analysis, &node.value);
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
        let var_type = resolving::types::expr(self.analysis, &node.expr);
        let var_ident = Ident::new(IdentSource::Var(node.name.id), var_type);
        self.analysis.idents.insert(node.name.id, var_ident);
        self.add_variable(&node.name);
    }

    fn exit_fn_call(&mut self, node: &AstFnCall) {
        if let Some(arg_type_ids) = resolving::types::fn_args(self.analysis, node) {
            if let Some(fn_) = resolving::items::fn_(self.analysis, node, &arg_type_ids) {
                let fn_ident =
                    Ident::new(IdentSource::Fn(fn_.id.clone()), fn_.return_type_id.clone());
                self.analysis.idents.insert(node.name.id, fn_ident);
            }
        }
    }

    fn exit_expr(&mut self, node: &AstExpr) {
        if self.is_const_context {
            return;
        }
        if let AstExprRoot::Ident(value) = &node.root {
            if let Some(&id) = self.variables.get(&value.label) {
                let var_type = self
                    .analysis
                    .idents
                    .get(&id)
                    .and_then(|var| var.type_id.clone());
                let var_ident = Ident::new(IdentSource::Var(id), var_type);
                self.analysis.idents.insert(value.id, var_ident);
            } else if let Some(buffer) = resolving::items::buffer(self.analysis, value) {
                let buffer_type = resolving::types::buffer(self.analysis, &buffer.id)
                    .map(|type_| type_.id.clone());
                let buffer_source = IdentSource::Buffer(buffer.id.clone());
                let buffer_ident = Ident::new(buffer_source, buffer_type);
                self.analysis.idents.insert(value.id, buffer_ident);
            }
        }
    }
}
