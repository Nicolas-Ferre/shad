use crate::compilation::index::NodeIndex;
use crate::compilation::node::{Node, NodeConfig};
use crate::language::items::buffer::BufferItem;
use crate::language::items::compute::{InitItem, RunItem};
use crate::language::items::Root;
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
    pub(crate) fn new(roots: &HashMap<PathBuf, Root>, index: &NodeIndex, root_path: &Path) -> Self {
        Self {
            buffers: roots
                .values()
                .flat_map(|root| root.items.iter().filter_map(|item| item.as_buffer()))
                .map(|buffer| (buffer.item_path(root_path), Buffer::new(buffer, index)))
                .collect(),
            init_shaders: Self::sorted_buffers(roots, index)
                .into_iter()
                .map(|item| Shader::from_buffer_item(item, index, root_path))
                .chain(roots.values().flat_map(|root| {
                    root.items
                        .iter()
                        .filter_map(|item| item.as_init())
                        .map(|item| Shader::from_init_item(item, index, root_path))
                }))
                .collect(),
            run_shaders: roots
                .values()
                .flat_map(|root| {
                    root.items
                        .iter()
                        .filter_map(|item| item.as_run())
                        .map(|item| Shader::from_run_item(item, index, root_path))
                })
                .collect(),
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
                if let Some(source) = (source as &dyn Any).downcast_ref::<BufferItem>() {
                    graph.add_edge(source, buffer, ());
                }
            }
        }
        petgraph::algo::toposort(&graph, None).expect("internal error: buffer cycle detected")
    }
}

/// A buffer definition.
#[derive(Debug)]
pub struct Buffer {
    /// The buffer size in bytes.
    pub size_bytes: u64,
    /// The buffer type name in Shad.
    pub type_name: String,
}

impl Buffer {
    fn new(item: &BufferItem, index: &NodeIndex) -> Self {
        Self {
            size_bytes: 4,
            type_name: item
                .expr_type(index)
                .expect("internal error: failed to calculate buffer type"),
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
    fn from_buffer_item(item: &BufferItem, index: &NodeIndex, root_path: &Path) -> Self {
        let mut ctx = TranspilationContext {
            index,
            next_binding: 0,
        };
        Self {
            code: item.transpile_shader(&mut ctx),
            buffers: Self::find_buffers(item, index, root_path)
                .into_iter()
                .chain([item.item_path(root_path)])
                .collect(),
        }
    }

    fn from_init_item(item: &InitItem, index: &NodeIndex, root_path: &Path) -> Self {
        let mut ctx = TranspilationContext {
            index,
            next_binding: 0,
        };
        Self {
            code: item.transpile_shader(&mut ctx),
            buffers: Self::find_buffers(item, index, root_path),
        }
    }

    fn from_run_item(item: &RunItem, index: &NodeIndex, root_path: &Path) -> Self {
        let mut ctx = TranspilationContext {
            index,
            next_binding: 0,
        };
        Self {
            code: item.transpile_shader(&mut ctx),
            buffers: Self::find_buffers(item, index, root_path),
        }
    }

    fn find_buffers(item: &impl Node, index: &NodeIndex, root_path: &Path) -> Vec<String> {
        item.nested_sources(index)
            .iter()
            .filter_map(|source| (*source as &dyn Any).downcast_ref::<BufferItem>())
            .map(|buffer| buffer.item_path(root_path))
            .collect()
    }
}

pub(crate) struct TranspilationContext<'a> {
    pub(crate) index: &'a NodeIndex,
    next_binding: u32,
}

impl TranspilationContext<'_> {
    pub(crate) fn next_binding(&mut self) -> u32 {
        let binding = self.next_binding;
        self.next_binding += 1;
        binding
    }
}
