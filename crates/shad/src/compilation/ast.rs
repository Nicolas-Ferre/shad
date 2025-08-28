use crate::compilation::{FILE_EXT, PRELUDE_PATH};
use crate::config::scripts::ScriptContext;
use crate::config::{scripts, KindConfig};
use derive_where::derive_where;
use itertools::Itertools;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::{iter, mem};

#[derive_where(PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct AstNode {
    pub id: u32,
    #[derive_where(skip)]
    pub parent_ids: Vec<u32>,
    #[derive_where(skip)]
    pub children: Vec<Rc<AstNode>>,
    #[derive_where(skip)]
    pub kind_name: String,
    #[derive_where(skip)]
    pub kind_config: Rc<KindConfig>,
    #[derive_where(skip)]
    pub slice: String,
    #[derive_where(skip)]
    pub span: Range<usize>,
    #[derive_where(skip)]
    pub path: PathBuf,
}

// coverage: off (no need to test)
impl Debug for AstNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AstNode")
            .field("kind_name", &self.kind_name)
            .field("slice", &self.slice)
            .field("children", &self.children)
            .finish_non_exhaustive()
    }
}
// coverage: on

impl AstNode {
    pub(crate) fn span(&self) -> Range<usize> {
        self.span.clone()
    }

    pub(crate) fn child(&self, child_name: &str) -> &Rc<Self> {
        self.child_option(child_name)
            .expect("internal error: node child not found")
    }

    pub(crate) fn child_option(&self, child_name: &str) -> Option<&Rc<Self>> {
        self.children
            .iter()
            .find(|child| child.kind_name == child_name)
    }

    pub(crate) fn nested_children(&self, child_name: &str) -> Vec<&Rc<Self>> {
        let mut children = vec![];
        self.scan(&mut |scanned| {
            if child_name.contains(&scanned.kind_name) {
                children.push(scanned);
                return true;
            }
            false
        });
        children
    }

    pub(crate) fn nested_children_except(
        &self,
        child_names: &[String],
        stop_child_names: Option<&[String]>,
    ) -> Vec<&Rc<Self>> {
        let mut children = vec![];
        self.scan(&mut |scanned| {
            if stop_child_names.is_some_and(|names| names.contains(&scanned.kind_name)) {
                return true;
            } else if child_names.contains(&scanned.kind_name) {
                children.push(scanned);
                return true;
            }
            false
        });
        children
    }

    pub(crate) fn scan<'a>(&'a self, f: &mut impl FnMut(&'a Rc<Self>) -> bool) {
        for child in &self.children {
            child.scan_inner(f);
        }
    }

    fn scan_inner<'a>(self: &'a Rc<Self>, f: &mut impl FnMut(&'a Rc<Self>) -> bool) {
        if f(self) {
            return;
        }
        for child in &self.children {
            child.scan_inner(f);
        }
    }

    pub(crate) fn type_(self: &Rc<Self>, ctx: &ScriptContext) -> Option<String> {
        self.type_inner(ctx, 0)
    }

    pub(crate) fn key(self: &Rc<Self>) -> String {
        self.kind_config
            .index_key
            .iter()
            .map(|key_part| {
                if let Some(child_kind) = &key_part.child {
                    self.child(child_kind).slice.clone()
                } else if let Some(nested_kind) = &key_part.nested {
                    let mut key_parts = vec![];
                    self.scan(&mut |scanned| {
                        if &scanned.kind_name == nested_kind {
                            key_parts.push(scanned.slice.clone());
                            true
                        } else {
                            false
                        }
                    });
                    key_parts.join(
                        key_part
                            .separator
                            .as_ref()
                            .expect("internal error: missing separator for `nested`"),
                    )
                } else if let Some(string) = &key_part.string {
                    string.clone()
                } else {
                    unreachable!("index key config should be valid");
                }
            })
            .join("")
    }

    pub(crate) fn source_key(self: &Rc<Self>, ctx: &ScriptContext) -> Option<String> {
        let script = &self.kind_config.index_key_source.as_ref()?.key;
        scripts::compile_and_run(script, self, ctx)
    }

    pub(crate) fn source<'a>(self: &Rc<Self>, ctx: &'a ScriptContext) -> Option<&'a Rc<Self>> {
        let prelude_path = PRELUDE_PATH.into();
        let key = self.source_key(ctx)?;
        for criteria in &self.kind_config.index_key_source.as_ref()?.criteria {
            let parent_id = self.parent_ids.last().copied().unwrap_or(0);
            let found_source = ctx.asts[&self.path]
                .index
                .indexed_lookup_paths
                .iter()
                .filter_map(|current_path| {
                    ctx.asts
                        .get(current_path)?
                        .index
                        .indexed_nodes
                        .get(&key)
                        .map(|nodes| (nodes, current_path))
                })
                .chain(
                    ctx.asts[&prelude_path]
                        .index
                        .indexed_nodes
                        .get(&key)
                        .map(|nodes| (nodes, &prelude_path)),
                )
                .flat_map(|(nodes, current_path)| {
                    nodes.iter().map(move |node| (node, current_path)).rev()
                })
                .find(|(node, current_path)| {
                    let node_parent_id = node.parent_ids.last().copied().unwrap_or(0);
                    let is_node_root_child = node.parent_ids.len() == 1;
                    let is_in_allowed_sibling = criteria.allowed_siblings.iter().any(|sibling| {
                        self.parent_ids.get(sibling.parent_index).is_some_and(|id| {
                            sibling
                                .child_offsets
                                .iter()
                                .any(|offset| node.parent_ids.contains(&(id + offset)))
                        })
                    });
                    (criteria.can_be_after || node.id < parent_id || &&self.path != current_path)
                        && node.kind_name == criteria.kind
                        && (is_in_allowed_sibling
                            || is_node_root_child
                            || self.parent_ids.contains(&node_parent_id))
                })
                .map(|(node, _)| node);
            if found_source.is_some() {
                return found_source;
            }
        }
        None
    }

    pub(crate) fn nested_sources<'a>(
        self: &'a Rc<Self>,
        ctx: &'a ScriptContext,
    ) -> Vec<&'a Rc<Self>> {
        let mut sources = vec![];
        let mut registered_source_ids = HashSet::new();
        let mut sources_to_process: HashMap<_, _> = iter::once((self.id, self)).collect();
        while !sources_to_process.is_empty() {
            for node in mem::take(&mut sources_to_process).into_values() {
                if registered_source_ids.contains(&node.id) {
                    continue;
                }
                registered_source_ids.insert(node.id);
                for source in node.direct_nested_sources(ctx) {
                    sources.push(source);
                    sources_to_process.insert(source.id, source);
                }
            }
        }
        sources
            .into_iter()
            .unique_by(|node| node.id)
            .sorted_by_key(|node| node.id)
            .collect()
    }

    pub(crate) fn import_path(self: &Rc<Self>, root_path: &Path) -> PathBuf {
        let config = self
            .kind_config
            .import_path
            .as_ref()
            .expect("internal error: missing `import_path` config property");
        let mut segments = vec![];
        self.scan(&mut |scanned| {
            if scanned.kind_name == config.parent || scanned.kind_name == config.segment {
                segments.push(scanned);
                true
            } else {
                false
            }
        });
        let mut path = if segments[0].kind_name == config.parent {
            self.path.clone()
        } else {
            root_path.to_path_buf()
        };
        for segment in &segments {
            if segment.kind_name == config.parent {
                path = path.parent().unwrap_or(&path).to_path_buf();
            } else {
                path.push(&segment.slice);
            }
        }
        path.set_extension(FILE_EXT);
        path
    }

    pub(crate) fn item_path(&self, child_ident: &str, root_path: &Path) -> String {
        format!(
            "{}.{}",
            self.path
                .strip_prefix(root_path)
                .expect("internal error: invalid root path")
                .with_extension("")
                .components()
                .map(|component| component.as_os_str().to_string_lossy())
                .join("."),
            self.child(child_ident).slice
        )
    }

    fn direct_nested_sources<'a>(self: &Rc<Self>, ctx: &'a ScriptContext) -> Vec<&'a Rc<Self>> {
        let mut sources = vec![];
        self.scan(&mut |scanned| {
            if let Some(source) = scanned.source(ctx) {
                sources.push(source);
            }
            false
        });
        sources
    }

    fn type_inner(self: &Rc<Self>, ctx: &ScriptContext, depth: u32) -> Option<String> {
        if depth > 1000 {
            // prevent stack overflow in case of circular dependency
            return None;
        }
        let type_ = if let Some(name) = &self.kind_config.type_resolution.name {
            Some(name.clone())
        } else if let Some(child_kind) = &self.kind_config.type_resolution.child_slice {
            Some(self.child(child_kind).slice.clone())
        } else if !self.kind_config.type_resolution.source_children.is_empty() {
            if let Some(source_source) = self.source(ctx) {
                self.kind_config
                    .type_resolution
                    .source_children
                    .iter()
                    .find_map(|source_child| {
                        source_source
                            .child_option(source_child)?
                            .type_inner(ctx, depth + 1)
                    })
            } else {
                None
            }
        } else {
            self.children
                .iter()
                .find_map(|child| child.type_inner(ctx, depth + 1))
        };
        type_.or_else(|| self.kind_config.type_resolution.default_name.clone())
    }
}
