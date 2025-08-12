use crate::config::{Config, ShaderConfig};
use crate::config::transpilation;
use crate::{AstNode, AstNodeInner, FileAst};
use itertools::Itertools;
use regex::Regex;
use std::collections::HashMap;
use std::ops::Deref;
use std::path::{Path, PathBuf};

pub(crate) fn transpile_asts(
    config: &Config,
    asts: &HashMap<PathBuf, FileAst>,
    root_path: &Path,
) -> Program {
    Program {
        buffers: asts
            .iter()
            .flat_map(|(path, ast)| ast.root.children().map(move |node| (node, path)))
            .filter_map(|(node, path)| {
                node.kind_config
                    .buffer
                    .as_ref()
                    .map(|config| (node, path, config))
            })
            .map(|(node, path, config)| {
                (
                    node.item_path(&config.ident, path, root_path),
                    Buffer { size_bytes: 4 },
                )
            })
            .collect(),
        init_shaders: asts
            .keys()
            .sorted_unstable()
            .flat_map(|path| {
                asts[path].root.children().filter_map(|node| {
                    Some(transpile_shader(
                        config,
                        asts,
                        root_path,
                        node,
                        path,
                        node.kind_config.init_shader.as_ref()?,
                    ))
                })
            })
            .collect(),
        run_shaders: asts
            .keys()
            .sorted_unstable()
            .flat_map(|path| {
                asts[path].root.children().filter_map(|node| {
                    Some(transpile_shader(
                        config,
                        asts,
                        root_path,
                        node,
                        path,
                        node.kind_config.run_shader.as_ref()?,
                    ))
                })
            })
            .collect(),
    }
}

pub(crate) fn transpile_shader(
    config: &Config,
    asts: &HashMap<PathBuf, FileAst>,
    root_path: &Path,
    node: &AstNode,
    path: &Path,
    shader_config: &ShaderConfig,
) -> Shader {
    let ctx = &mut Context::new(asts, config, path);
    Shader {
        code: template_code(ctx, node, &shader_config.transpilation),
        buffers: node
            .nested_sources(asts, path)
            .into_iter()
            .map(Deref::deref)
            .chain([node])
            .filter_map(|node| {
                node.kind_config
                    .buffer
                    .as_ref()
                    .map(|buffer| (node, buffer))
            })
            .map(|(node, config)| node.item_path(&config.ident, path, root_path))
            .collect(),
    }
}

pub(crate) fn node_code(ctx: &mut Context<'_>, node: &AstNode) -> String {
    if let AstNodeInner::Repeated(children) = &node.children {
        children
            .iter()
            .map(|child| template_code(ctx, child, &child.kind_config.transpilation))
            .join("\n")
    } else {
        template_code(ctx, node, &node.kind_config.transpilation)
    }
}

fn template_code(ctx: &mut Context<'_>, node: &AstNode, template: &str) -> String {
    let mut code = template.to_string();
    let placeholders = ctx.placeholders_mut(&node.kind_name).clone();
    for placeholder in &placeholders {
        if code.contains(&placeholder.raw) {
            let placeholder_code = transpilation::run(ctx, node, placeholder);
            code = code.replace(&placeholder.raw, &placeholder_code);
        }
    }
    code
}

#[derive(Debug)]
pub struct Program {
    pub buffers: HashMap<String, Buffer>,
    pub init_shaders: Vec<Shader>,
    pub run_shaders: Vec<Shader>,
}

#[derive(Debug)]
pub struct Buffer {
    pub size_bytes: u64,
}

#[derive(Debug)]
pub struct Shader {
    pub code: String,
    pub buffers: Vec<String>,
}

pub(crate) struct Context<'a> {
    pub(crate) asts: &'a HashMap<PathBuf, FileAst>,
    pub(crate) path: &'a Path,
    pub(crate) next_binding: u32,
    kind_placeholders: HashMap<String, Vec<KindPlaceholder>>,
}

impl<'a> Context<'a> {
    fn new(asts: &'a HashMap<PathBuf, FileAst>, config: &'a Config, path: &'a Path) -> Self {
        let placeholder_regex =
            Regex::new(r"\{\{([^}]+)}}").expect("internal error: invalid placeholder regex");
        Self {
            asts,
            path,
            next_binding: 0,
            kind_placeholders: config
                .kinds
                .iter()
                .map(|(name, kind)| {
                    (
                        name.clone(),
                        kind.init_shader
                            .iter()
                            .map(|shader| &shader.transpilation)
                            .chain(kind.run_shader.iter().map(|shader| &shader.transpilation))
                            .chain([&kind.transpilation])
                            .flat_map(|transpilation| {
                                placeholder_regex
                                    .captures_iter(transpilation)
                                    .map(|cap| KindPlaceholder::new(&cap[1]))
                            })
                            .collect(),
                    )
                })
                .collect(),
        }
    }

    pub(crate) fn generate_binding(&mut self) -> u32 {
        let binding = self.next_binding;
        self.next_binding += 1;
        binding
    }

    fn placeholders_mut(&mut self, kind_name: &str) -> &mut Vec<KindPlaceholder> {
        self.kind_placeholders
            .get_mut(kind_name)
            .expect("internal error: missing kind placeholders")
    }
}

#[derive(Debug, Clone)]
pub(crate) struct KindPlaceholder {
    pub(crate) raw: String,
    pub(crate) name: String,
    pub(crate) params: Vec<String>,
}

impl KindPlaceholder {
    fn new(content: &str) -> Self {
        let mut parts = content.split(':');
        Self {
            raw: format!("{{{{{content}}}}}"),
            name: parts
                .next()
                .expect("internal error: missing placeholder name")
                .to_string(),
            params: parts.map(str::to_string).collect(),
        }
    }
}
