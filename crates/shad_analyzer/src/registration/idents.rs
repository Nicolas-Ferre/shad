use crate::registration::constants::{Constant, ConstantId};
use crate::{errors, resolver, Analysis, Buffer, BufferId, FnId, Function, TypeId};
use fxhash::FxHashMap;
use shad_parser::{
    AstBufferItem, AstConstItem, AstExpr, AstExprRoot, AstFnCall, AstFnItem, AstFnParam,
    AstGpuGenericParam, AstGpuName, AstIdent, AstItem, AstVarDefinition, Visit,
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
    /// A constant.
    Constant(ConstantId),
    /// A buffer.
    Buffer(BufferId),
    /// A variable.
    Var(u64),
    /// A function.
    Fn(FnId),
    /// A field.
    Field,
    /// A generic type.
    GenericType,
}

pub(crate) fn register(analysis: &mut Analysis) {
    register_structs(analysis);
    register_constant_init(analysis);
    register_constant_types(analysis);
    register_buffer_init(analysis);
    register_buffer_types(analysis);
    register_run_blocks(analysis);
    register_fns(analysis);
}

fn register_structs(analysis: &mut Analysis) {
    for type_ in analysis.types.clone().values() {
        for field in &type_.fields {
            let ident = Ident::new(IdentSource::Field, field.type_id.clone());
            analysis.idents.insert(field.name.id, ident);
        }
        let ast_and_name = type_
            .ast
            .as_ref()
            .and_then(|ast| ast.gpu_qualifier.as_ref().map(|gpu| (ast, gpu)))
            .and_then(|(ast, gpu)| gpu.name.as_ref().map(|name| (ast, name)));
        if let Some((ast, name)) = &ast_and_name {
            register_gpu_name(analysis, &ast.name.span.module.name, name);
        }
    }
}

fn register_constant_init(analysis: &mut Analysis) {
    for constant in analysis.constants.clone().values() {
        let module = &constant.ast.name.span.module.name;
        IdentRegistration::new(analysis, module, true, true).visit_const_item(&constant.ast);
    }
}

fn register_constant_types(analysis: &mut Analysis) {
    let constant_count = count_constants(analysis);
    let mut typed_constant_count = 0;
    let mut last_typed_constant_count = 0;
    let constants = analysis.constants.clone();
    while typed_constant_count < constant_count {
        for constant in constants.values() {
            if analysis.idents[&constant.ast.name.id].type_id.is_none() {
                let module = &constant.ast.name.span.module.name;
                IdentRegistration::new(analysis, module, true, false)
                    .visit_const_item(&constant.ast);
            }
        }
        typed_constant_count = count_typed_constants(analysis);
        if typed_constant_count == last_typed_constant_count {
            break; // recursive constant init
        }
        last_typed_constant_count = typed_constant_count;
    }
}

fn register_buffer_init(analysis: &mut Analysis) {
    let asts = mem::take(&mut analysis.asts);
    for (module, ast) in &asts {
        for item in &ast.items {
            if let AstItem::Buffer(buffer) = item {
                IdentRegistration::new(analysis, module, false, true).visit_buffer_item(buffer);
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
                IdentRegistration::new(analysis, module, false, false)
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
        IdentRegistration::new(analysis, &block.module, false, true).visit_run_item(&block.ast);
    }
    analysis.run_blocks = blocks;
}

fn register_fns(analysis: &mut Analysis) {
    for fn_ in analysis.fns.clone().into_values() {
        IdentRegistration::new(analysis, &fn_.ast.name.span.module.name, false, true)
            .visit_fn_item(&fn_.ast);
        let name = fn_
            .ast
            .gpu_qualifier
            .as_ref()
            .and_then(|gpu| gpu.name.as_ref());
        if let Some(name) = name {
            register_gpu_name(analysis, &fn_.id.module, name);
        }
    }
}

fn register_gpu_name(analysis: &mut Analysis, module: &str, name: &AstGpuName) {
    for param in &name.generics {
        if let AstGpuGenericParam::Ident(param) = param {
            let type_id = resolver::type_or_add_error(analysis, module, param);
            let ident = Ident::new(IdentSource::GenericType, type_id);
            analysis.idents.insert(param.id, ident);
        }
    }
}

fn count_constants(analysis: &Analysis) -> usize {
    analysis
        .idents
        .values()
        .filter(|e| matches!(e.source, IdentSource::Constant(_)))
        .count()
}

fn count_typed_constants(analysis: &Analysis) -> usize {
    analysis
        .idents
        .values()
        .filter(|e| matches!(e.source, IdentSource::Constant(_)))
        .filter(|e| e.type_id.is_some())
        .count()
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
    are_errors_enabled: bool,
    variables: FxHashMap<String, u64>,
}

impl<'a> IdentRegistration<'a> {
    pub(crate) fn new(
        analysis: &'a mut Analysis,
        module: &'a str,
        is_const_context: bool,
        are_errors_enabled: bool,
    ) -> Self {
        Self {
            analysis,
            module,
            is_const_context,
            are_errors_enabled,
            variables: FxHashMap::default(),
        }
    }

    fn add_variable(&mut self, ident: &AstIdent) {
        self.variables.insert(ident.label.clone(), ident.id);
    }

    fn find_fn(&self, call: &AstFnCall, arg_types: &[TypeId]) -> Option<(FnId, &Function)> {
        self.analysis
            .visible_modules
            .get(self.module)
            .into_iter()
            .flatten()
            .filter_map(|module| {
                let id = FnId {
                    module: module.clone(),
                    name: call.name.label.clone(),
                    param_types: arg_types.iter().map(|type_| Some(type_.clone())).collect(),
                    param_count: arg_types.len(),
                    is_generic: false,
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

    fn find_constant(&self, name: &str) -> Option<(ConstantId, &Constant)> {
        self.analysis
            .visible_modules
            .get(self.module)
            .into_iter()
            .flatten()
            .filter_map(|module| {
                let id = ConstantId {
                    module: module.clone(),
                    name: name.into(),
                };
                self.analysis.constants.get(&id).map(|buffer| (id, buffer))
            })
            .find(|(buffer_id, buffer)| buffer.ast.is_pub || buffer_id.module == self.module)
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
        let type_id = resolver::type_(self.analysis, self.module, &param.type_).ok();
        let ident = Ident::new(IdentSource::Var(param.name.id), type_id);
        self.analysis.idents.insert(param.name.id, ident);
        self.add_variable(&param.name);
    }

    fn register_variable(&mut self, variable: &AstIdent) -> Option<TypeId> {
        if let Some(&id) = self.variables.get(&variable.label) {
            let var_type = self
                .analysis
                .idents
                .get(&id)
                .and_then(|var| var.type_id.clone());
            let var_ident = Ident::new(IdentSource::Var(id), var_type.clone());
            self.analysis.idents.insert(variable.id, var_ident);
            var_type
        } else if let (Some((buffer_id, _)), false) =
            (self.find_buffer(&variable.label), self.is_const_context)
        {
            let buffer_type =
                resolver::buffer_type(self.analysis, &buffer_id).map(|type_| type_.id.clone());
            let buffer_ident = Ident::new(IdentSource::Buffer(buffer_id), buffer_type.clone());
            self.analysis.idents.insert(variable.id, buffer_ident);
            buffer_type
        } else if let Some((constant_id, _)) = self.find_constant(&variable.label) {
            let constant_type =
                resolver::constant_type(self.analysis, &constant_id).map(|type_| type_.id.clone());
            let constant_ident =
                Ident::new(IdentSource::Constant(constant_id), constant_type.clone());
            self.analysis.idents.insert(variable.id, constant_ident);
            constant_type
        } else if self.are_errors_enabled {
            let error = errors::variables::not_found(variable);
            self.analysis.errors.push(error);
            None
        } else {
            None
        }
    }
}

impl Visit for IdentRegistration<'_> {
    fn enter_fn_item(&mut self, node: &AstFnItem) {
        self.register_fn_item(node);
        for param in &node.params {
            self.register_fn_param(param);
        }
    }

    fn exit_const_item(&mut self, node: &AstConstItem) {
        let constant_type = resolver::expr_type(self.analysis, &node.value);
        let constant_ident = Ident::new(
            IdentSource::Constant(ConstantId {
                module: self.module.into(),
                name: node.name.label.clone(),
            }),
            constant_type,
        );
        self.analysis.idents.insert(node.name.id, constant_ident);
    }

    fn exit_buffer_item(&mut self, node: &AstBufferItem) {
        let buffer_type = resolver::expr_type(self.analysis, &node.value);
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
        let var_type = resolver::expr_type(self.analysis, &node.expr);
        let var_ident = Ident::new(IdentSource::Var(node.name.id), var_type);
        self.analysis.idents.insert(node.name.id, var_ident);
        self.add_variable(&node.name);
    }

    fn exit_fn_call(&mut self, node: &AstFnCall) {
        if let Some(arg_type_ids) = node
            .args
            .iter()
            .map(|node| resolver::expr_type(self.analysis, &node.value))
            .collect::<Option<Vec<_>>>()
        {
            if let Some((fn_id, fn_)) = self.find_fn(node, &arg_type_ids) {
                let fn_ident = Ident::new(IdentSource::Fn(fn_id), fn_.return_type_id.clone());
                self.analysis.idents.insert(node.name.id, fn_ident);
            } else if self.are_errors_enabled {
                let error = errors::functions::not_found(node, &arg_type_ids);
                self.analysis.errors.push(error);
            }
        }
    }

    fn exit_expr(&mut self, node: &AstExpr) {
        let mut last_type = match &node.root {
            AstExprRoot::Ident(value) => self.register_variable(value),
            AstExprRoot::FnCall(value) => self
                .analysis
                .idents
                .get(&value.name.id)
                .and_then(|ident| ident.type_id.clone()),
            AstExprRoot::Literal(literal) => Some(resolver::literal_type(literal)),
        };
        for field in &node.fields {
            let Some(current_type) = last_type.clone() else {
                return;
            };
            let type_field = self.analysis.types[&current_type]
                .fields
                .iter()
                .filter(|type_field| {
                    type_field.is_pub || current_type.module.as_deref() == Some(self.module)
                })
                .find(|type_field| type_field.name.label == field.label);
            if type_field.is_none() && self.are_errors_enabled {
                let error = errors::types::field_not_found(field, &current_type);
                self.analysis.errors.push(error);
            }
            last_type = type_field.and_then(|field| field.type_id.clone());
            let buffer_ident = Ident::new(IdentSource::Field, last_type.clone());
            self.analysis.idents.insert(field.id, buffer_ident);
        }
    }
}
