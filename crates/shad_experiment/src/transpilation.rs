use crate::config::Config;
use crate::functions::transpilation;
use crate::{AstNode, AstNodeInner, FileAst};
use itertools::Itertools;
use regex::Regex;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

// TODO: add path index to var names -> add path into each node and refactor
pub(crate) fn transpile_asts(config: &Config, asts: &HashMap<PathBuf, FileAst>) -> Vec<String> {
    asts.keys()
        .sorted_unstable()
        .flat_map(|path| transpile(config, path, asts))
        .collect()
}

fn transpile(config: &Config, path: &Path, asts: &HashMap<PathBuf, FileAst>) -> Vec<String> {
    let root_nodes = match &asts[path].root.inner {
        AstNodeInner::Repeated(children) => children,
        AstNodeInner::Sequence(_) | AstNodeInner::Terminal => {
            unreachable!("root node should be repeated")
        }
    };
    let init_shaders = root_nodes
        .iter()
        .filter_map(|node| init_shader_code(config, path, asts, node));
    let run_shaders = root_nodes
        .iter()
        .filter_map(|node| run_shader_code(config, path, asts, node));
    init_shaders.chain(run_shaders).collect()
}

fn init_shader_code(
    config: &Config,
    path: &Path,
    asts: &HashMap<PathBuf, FileAst>,
    node: &AstNode,
) -> Option<String> {
    let ctx = &mut Context::new(asts, config, path);
    node.kind_config
        .init_shader
        .as_ref()
        .map(|shader| template_code(ctx, node, &shader.transpilation))
}

fn run_shader_code(
    config: &Config,
    path: &Path,
    asts: &HashMap<PathBuf, FileAst>,
    node: &AstNode,
) -> Option<String> {
    let ctx = &mut Context::new(asts, config, path);
    node.kind_config
        .run_shader
        .as_ref()
        .map(|shader| template_code(ctx, node, &shader.transpilation))
}

pub(crate) fn node_code(ctx: &mut Context<'_>, node: &AstNode) -> String {
    if let AstNodeInner::Repeated(children) = &node.inner {
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
