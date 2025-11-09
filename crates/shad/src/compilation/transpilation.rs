use crate::compilation::index::NodeIndex;
use crate::compilation::node::Node;
use crate::language::items::buffer::BufferItem;
use crate::language::items::compute::{InitItem, RunItem};
use crate::language::items::Root;
use itertools::Itertools;
use petgraph::graphmap::DiGraphMap;
use std::any::Any;
use std::collections::HashMap;
use std::hash::RandomState;
use std::path::{Path, PathBuf};

/// A compiled Shad program.
#[derive(Debug)]
pub struct Program {
    /// The program GPU buffers.
    pub buffers: HashMap<String, Buffer>,
    /// The program `init` shaders, run only once at module creation.
    pub init_shaders: Vec<Shader>,
    /// The program `run` shaders, run at each frame.
    pub run_shaders: Vec<Shader>,
}

impl Program {
    pub(crate) fn new(
        roots: &HashMap<PathBuf, Root>,
        index: &NodeIndex,
        root_path: &Path,
        next_node_id: u32,
    ) -> Self {
        let mut ctx = TranspilationContext {
            index,
            inline_state: InlineState {
                is_inlined: false,
                is_returning_ref: false,
                return_var_id: None,
                returned_ref: None,
            },
            generated_stmts: vec![],
            block_inline_mappings: vec![],
            root_path,
            next_binding: 0,
            next_node_id,
        };
        Self {
            buffers: Self::sorted_roots(roots)
                .flat_map(|root| root.items.iter().filter_map(|item| item.as_buffer()))
                .map(|buffer| (buffer.item_path(root_path), Buffer::new(buffer, index)))
                .collect(),
            init_shaders: Self::sorted_buffers(roots, index)
                .into_iter()
                .map(|item| Shader::from_buffer_item(item, &mut ctx))
                .collect::<Vec<_>>()
                .into_iter()
                .chain(
                    Self::sorted_roots(roots)
                        .flat_map(|root| root.items.iter().filter_map(|item| item.as_init()))
                        .enumerate()
                        .sorted_by_key(|(position, item)| (-item.priority(index), *position))
                        .map(|(_, item)| Shader::from_init_item(item, &mut ctx))
                        .collect::<Vec<_>>(),
                )
                .collect(),
            run_shaders: Self::sorted_roots(roots)
                .flat_map(|root| root.items.iter().filter_map(|item| item.as_run()))
                .enumerate()
                .sorted_by_key(|(position, item)| (-item.priority(index), *position))
                .map(|(_, item)| Shader::from_run_item(item, &mut ctx))
                .collect::<Vec<_>>(),
        }
    }

    fn sorted_buffers<'a>(
        roots: &'a HashMap<PathBuf, Root>,
        index: &'a NodeIndex,
    ) -> Vec<&'a BufferItem> {
        let mut graph = DiGraphMap::<&BufferItem, (), RandomState>::new();
        let buffers = roots
            .values()
            .flat_map(|root| root.items.iter().filter_map(|item| item.as_buffer()));
        for buffer in buffers {
            graph.add_node(buffer);
            for source in buffer.nested_sources(index) {
                let source_node = source.as_node();
                if let Some(source) = (source_node as &dyn Any).downcast_ref::<BufferItem>() {
                    graph.add_edge(source, buffer, ());
                }
            }
        }
        petgraph::algo::toposort(&graph, None).expect("internal error: buffer cycle detected")
    }

    fn sorted_roots(roots: &HashMap<PathBuf, Root>) -> impl Iterator<Item = &Root> {
        roots
            .iter()
            .sorted_by_key(|(path, _)| *path)
            .map(|(_, root)| root)
    }
}

/// A buffer definition.
#[derive(Debug)]
pub struct Buffer {
    /// The buffer size in bytes.
    pub size_bytes: u32,
    /// The buffer type name in Shad.
    pub type_name: String,
}

impl Buffer {
    fn new(item: &BufferItem, index: &NodeIndex) -> Self {
        let type_ = item.buffer_type(index);
        Self {
            size_bytes: type_.size(index),
            type_name: type_.ident().slice.clone(),
        }
    }
}

/// A shader definition.
#[derive(Debug)]
pub struct Shader {
    /// The shader WGSL code.
    pub code: String,
    /// The buffers used by the shader.
    pub buffers: Vec<String>,
}

impl Shader {
    fn from_buffer_item(item: &BufferItem, ctx: &mut TranspilationContext<'_>) -> Self {
        ctx.next_binding = 0;
        Self {
            code: item.transpile_shader(ctx),
            buffers: Self::find_buffers(item, ctx)
                .into_iter()
                .chain([item.item_path(ctx.root_path)])
                .collect(),
        }
    }

    fn from_init_item(item: &InitItem, ctx: &mut TranspilationContext<'_>) -> Self {
        ctx.next_binding = 0;
        Self {
            code: item.transpile_shader(ctx),
            buffers: Self::find_buffers(item, ctx),
        }
    }

    fn from_run_item(item: &RunItem, ctx: &mut TranspilationContext<'_>) -> Self {
        ctx.next_binding = 0;
        Self {
            code: item.transpile_shader(ctx),
            buffers: Self::find_buffers(item, ctx),
        }
    }

    fn find_buffers(item: &impl Node, ctx: &TranspilationContext<'_>) -> Vec<String> {
        item.nested_sources(ctx.index)
            .iter()
            .filter_map(|source| (source.as_node() as &dyn Any).downcast_ref::<BufferItem>())
            .map(|buffer| buffer.item_path(ctx.root_path))
            .collect()
    }
}

#[derive(Debug)]
pub(crate) struct TranspilationContext<'a> {
    pub(crate) index: &'a NodeIndex,
    pub(crate) generated_stmts: Vec<String>,
    pub(crate) inline_state: InlineState,
    block_inline_mappings: Vec<HashMap<u32, String>>,
    root_path: &'a Path,
    next_binding: u32,
    next_node_id: u32,
}

impl TranspilationContext<'_> {
    pub(crate) fn next_binding(&mut self) -> u32 {
        let binding = self.next_binding;
        self.next_binding += 1;
        binding
    }

    pub(crate) fn next_node_id(&mut self) -> u32 {
        let id = self.next_node_id;
        self.next_node_id += 1;
        id
    }

    pub(crate) fn start_block(&mut self) {
        self.block_inline_mappings.push(HashMap::new());
    }

    pub(crate) fn end_block(&mut self) {
        self.block_inline_mappings.pop();
    }

    pub(crate) fn inline_mapping(&self, id: u32) -> Option<&str> {
        self.block_inline_mappings
            .iter()
            .rev()
            .find_map(|mapping| mapping.get(&id))
            .map(|mapping| &**mapping)
    }

    pub(crate) fn add_inline_mapping(&mut self, id: u32, mapping: impl Into<String>) {
        let last_block = self.block_inline_mappings.len() - 1;
        self.block_inline_mappings[last_block].insert(id, mapping.into());
    }
}

#[derive(Debug, Clone)]
pub(crate) struct InlineState {
    pub(crate) is_inlined: bool,
    pub(crate) is_returning_ref: bool,
    pub(crate) return_var_id: Option<u32>,
    pub(crate) returned_ref: Option<String>,
}
