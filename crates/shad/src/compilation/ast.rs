use crate::compilation::{FileAst, FILE_EXT};
use crate::config::KindConfig;
use itertools::Itertools;
use std::borrow::Cow;
use std::collections::HashMap;
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::slice::Iter;

#[derive(Debug)]
pub(crate) struct AstNode {
    pub id: u32,
    pub parent_ids: Vec<u32>,
    pub children: AstNodeInner,
    pub kind_name: String,
    pub kind_config: Rc<KindConfig>,
    pub slice: String,
    pub offset: usize,
    pub path: PathBuf,
}

impl AstNode {
    pub(crate) fn span(&self) -> Range<usize> {
        self.offset..self.offset + self.slice.len()
    }

    pub(crate) fn children(&self) -> NodeChildIterator<'_> {
        match &self.children {
            AstNodeInner::Sequence(children) => {
                NodeChildIterator::Sequence(self.kind_config.sequence.iter(), children)
            }
            AstNodeInner::Repeated(children) => NodeChildIterator::Repeated(children.iter()),
            AstNodeInner::Terminal => NodeChildIterator::Terminal,
        }
    }

    pub(crate) fn child(&self, child_name: &str) -> &Rc<Self> {
        self.child_option(child_name)
            .expect("internal error: node child not found")
    }

    pub(crate) fn child_option(&self, child_name: &str) -> Option<&Rc<Self>> {
        match &self.children {
            AstNodeInner::Sequence(children) => children.get(child_name),
            AstNodeInner::Repeated(_) | AstNodeInner::Terminal => {
                unreachable!("cannot retrieve child for non-sequence node")
            }
        }
    }

    #[allow(clippy::needless_lifetimes)]
    pub(crate) fn scan<'a>(self: &'a Rc<Self>, f: &mut impl FnMut(&'a Rc<Self>) -> bool) {
        if f(self) {
            return;
        }
        for child in self.children() {
            child.scan(f);
        }
    }

    pub(crate) fn type_(self: &Rc<Self>, asts: &HashMap<PathBuf, FileAst>) -> Option<String> {
        let type_ = if let Some(name) = &self.kind_config.type_resolution.name {
            Some(name.clone())
        } else if let Some(child_kind) = &self.kind_config.type_resolution.child_slice {
            Some(self.child(child_kind).slice.clone())
        } else if !self.kind_config.type_resolution.source_children.is_empty() {
            if let Some(source_source) = self.source(asts) {
                self.kind_config
                    .type_resolution
                    .source_children
                    .iter()
                    .find_map(|source_child| source_source.child_option(source_child)?.type_(asts))
            } else {
                None
            }
        } else {
            self.children().find_map(|child| child.type_(asts))
        };
        type_.or_else(|| self.kind_config.type_resolution.default_name.clone())
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

    pub(crate) fn source_key(self: &Rc<Self>, asts: &HashMap<PathBuf, FileAst>) -> Option<String> {
        let source_config = self.kind_config.index_key_source.as_ref()?;
        Some(
            source_config
                .key
                .iter()
                .map(|key_part| {
                    if key_part.slice {
                        Cow::from(&self.slice)
                    } else if let Some(string) = &key_part.string {
                        Cow::from(string)
                    } else if let Some(child_kind) = &key_part.slice_child {
                        Cow::from(&self.child(child_kind).slice)
                    } else if let Some(child_kind) = &key_part.type_nested_child {
                        let mut key_parts = vec![];
                        self.scan(&mut |scanned| {
                            if &scanned.kind_name == child_kind {
                                key_parts.push(
                                    scanned.type_(asts).unwrap_or_else(|| "<unknown>".into()),
                                );
                                true
                            } else {
                                false
                            }
                        });
                        Cow::from(key_parts.join(
                            key_part.separator.as_ref().expect(
                                "internal error: missing separator for `type_nested_child`",
                            ),
                        ))
                    } else {
                        unreachable!("index key source config should be valid");
                    }
                })
                .join(""),
        )
    }

    pub(crate) fn source<'a>(
        self: &Rc<Self>,
        asts: &'a HashMap<PathBuf, FileAst>,
    ) -> Option<&'a Rc<Self>> {
        let key = self.source_key(asts)?;
        for criteria in &self.kind_config.index_key_source.as_ref()?.criteria {
            let parent_id = self.parent_ids.last().copied().unwrap_or(0);
            let found_source = asts[&self.path]
                .index
                .indexed_lookup_paths
                .iter()
                .filter_map(|current_path| {
                    asts.get(current_path)?
                        .index
                        .indexed_nodes
                        .get(&key)
                        .map(|nodes| (nodes, current_path))
                })
                .flat_map(|(nodes, current_path)| {
                    nodes.iter().map(move |node| (node, current_path)).rev()
                })
                .find(|(node, current_path)| {
                    let node_parent_id = node.parent_ids.last().copied().unwrap_or(0);
                    let is_node_root_child = node.parent_ids.len() == 1;
                    let is_in_allowed_sibling = criteria.allowed_siblings.iter().any(|sibling| {
                        self.parent_ids.get(sibling.parent_index).is_some_and(|id| {
                            node.parent_ids.contains(&(id + sibling.child_offset))
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
        self: &Rc<Self>,
        asts: &'a HashMap<PathBuf, FileAst>,
    ) -> Vec<&'a Rc<Self>> {
        let mut sources = HashMap::new();
        let mut previous_source_count = usize::MAX;
        while sources.len() != previous_source_count {
            previous_source_count = sources.len();
            self.scan(&mut |scanned| {
                if let Some(source) = scanned.source(asts) {
                    let mut is_new = false;
                    sources.entry(source.id).or_insert_with(|| {
                        is_new = true;
                        source
                    });
                    if is_new {
                        sources.extend(
                            source
                                .nested_sources(asts)
                                .into_iter()
                                .map(|node| (node.id, node)),
                        );
                    }
                    true
                } else {
                    false
                }
            });
        }
        sources
            .into_values()
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
}

#[derive(Debug)]
pub(crate) enum AstNodeInner {
    Sequence(HashMap<String, Rc<AstNode>>),
    Repeated(Vec<Rc<AstNode>>),
    Terminal,
}

pub(crate) enum NodeChildIterator<'a> {
    Sequence(Iter<'a, String>, &'a HashMap<String, Rc<AstNode>>),
    Repeated(Iter<'a, Rc<AstNode>>),
    Terminal,
}

impl<'a> Iterator for NodeChildIterator<'a> {
    type Item = &'a Rc<AstNode>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            NodeChildIterator::Sequence(keys, children) => keys.next().map(|key| &children[key]),
            NodeChildIterator::Repeated(iter) => iter.next(),
            NodeChildIterator::Terminal => None,
        }
    }
}
