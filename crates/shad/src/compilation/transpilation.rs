use crate::compilation::ast::{AstNode, AstNodeInner};
use crate::compilation::FileAst;
use crate::config::scripts::{compile_and_run, ScriptContext};
use crate::config::{Config, ShaderConfig};
use itertools::Itertools;
use petgraph::graphmap::DiGraphMap;
use std::collections::HashMap;
use std::hash::RandomState;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::rc::Rc;

pub(crate) fn transpile_asts(
    config: &Rc<Config>,
    asts: &Rc<HashMap<PathBuf, FileAst>>,
    root_path: &Path,
) -> Program {
    let ctx = ScriptContext::new(config, asts, root_path);
    Program {
        buffers: asts
            .values()
            .flat_map(|ast| ast.root.children())
            .filter_map(|node| {
                node.kind_config
                    .buffer
                    .as_ref()
                    .map(|config| (node, config))
            })
            .map(|(node, config)| {
                (
                    node.item_path(&config.ident, root_path),
                    Buffer {
                        size_bytes: 4,
                        type_name: node
                            .type_(asts)
                            .expect("internal error: missing buffer type"),
                    },
                )
            })
            .collect(),
        init_shaders: sorted_buffers(&ctx)
            .into_iter()
            .filter_map(|node| {
                Some(transpile_shader(
                    &ctx,
                    node,
                    node.kind_config.buffer_init_shader.as_ref()?,
                ))
            })
            .chain(
                asts.values()
                    .sorted_unstable_by_key(|ast| &ast.root.path)
                    .flat_map(|ast| {
                        ast.root.children().filter_map(|node| {
                            Some(transpile_shader(
                                &ctx,
                                node,
                                node.kind_config.init_shader.as_ref()?,
                            ))
                        })
                    }),
            )
            .collect(),
        run_shaders: asts
            .values()
            .sorted_unstable_by_key(|ast| &ast.root.path)
            .flat_map(|ast| {
                ast.root.children().filter_map(|node| {
                    Some(transpile_shader(
                        &ctx,
                        node,
                        node.kind_config.run_shader.as_ref()?,
                    ))
                })
            })
            .collect(),
    }
}

pub(crate) fn transpile_node(ctx: &ScriptContext, node: &Rc<AstNode>) -> String {
    if let AstNodeInner::Repeated(children) = &node.children {
        children
            .iter()
            .map(|child| transpile_from_script(ctx, child, &child.kind_config.transpilation))
            .join("\n")
    } else {
        transpile_from_script(ctx, node, &node.kind_config.transpilation)
    }
}

fn sorted_buffers(ctx: &ScriptContext) -> Vec<&Rc<AstNode>> {
    let mut graph = DiGraphMap::<&Rc<AstNode>, (), RandomState>::new();
    let buffers = ctx.asts.values().flat_map(|ast| {
        ast.root
            .children()
            .filter(|node| node.kind_config.buffer_init_shader.is_some())
    });
    for item in buffers {
        graph.add_node(item);
        for source in item.nested_sources(&ctx.asts) {
            graph.add_edge(source, item, ());
        }
    }
    petgraph::algo::toposort(&graph, None).expect("internal error: buffer cycle detected")
}

fn transpile_shader(
    ctx: &ScriptContext,
    node: &Rc<AstNode>,
    shader_config: &ShaderConfig,
) -> Shader {
    ctx.next_binding.replace(0);
    Shader {
        code: transpile_from_script(ctx, node, &shader_config.transpilation),
        buffers: node
            .nested_sources(&ctx.asts)
            .into_iter()
            .map(Deref::deref)
            .chain([&**node])
            .filter_map(|node| {
                node.kind_config
                    .buffer
                    .as_ref()
                    .map(|buffer| (node, buffer))
            })
            .map(|(node, config)| node.item_path(&config.ident, &ctx.root_path))
            .collect(),
    }
}

fn transpile_from_script(ctx: &ScriptContext, node: &Rc<AstNode>, template: &str) -> String {
    compile_and_run::<String>(template, node, ctx)
        .expect("internal error: invalid transpilation script")
}

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

/// A buffer definition.
#[derive(Debug)]
pub struct Buffer {
    /// The buffer size in bytes.
    pub size_bytes: u64,
    /// The buffer type name in Shad.
    pub type_name: String,
}

/// A shader definition.
#[derive(Debug)]
pub struct Shader {
    /// The shader WGSL code.
    pub code: String,
    /// The buffers used by the shader.
    pub buffers: Vec<String>,
}
