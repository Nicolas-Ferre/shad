use crate::config::KindConfig;
use crate::{FileAst, FILE_EXT};
use itertools::Itertools;
use std::collections::HashMap;
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::slice::Iter;

#[derive(Debug)]
pub struct AstNode {
    pub id: u32,
    pub parent_ids: Vec<u32>,
    pub children: AstNodeInner,
    pub kind_name: String,
    pub kind_config: Rc<KindConfig>,
    pub slice: String,
    pub offset: usize,
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

    pub(crate) fn child(&self, child_name: &str) -> &Self {
        match &self.children {
            AstNodeInner::Sequence(children) => &children[child_name],
            AstNodeInner::Repeated(_) | AstNodeInner::Terminal => {
                unreachable!("cannot retrieve child for non-sequence node")
            }
        }
    }

    pub(crate) fn type_(&self, asts: &HashMap<PathBuf, FileAst>, path: &Path) -> Option<String> {
        if let Some(name) = &self.kind_config.type_resolution.name {
            Some(name.clone())
        } else if let Some(source_child) = &self.kind_config.type_resolution.source_child {
            if let Some(source_source) = self.source(asts, path) {
                source_source.child(source_child).type_(asts, path)
            } else {
                None
            }
        } else {
            self.children().find_map(|child| child.type_(asts, path))
        }
    }

    pub(crate) fn source<'a>(
        &self,
        asts: &'a HashMap<PathBuf, FileAst>,
        path: &Path,
    ) -> Option<&'a Rc<Self>> {
        let Some(source_node_config) = &self.kind_config.index_key_source else {
            return None;
        };
        let parent_id = self.parent_ids.last().copied().unwrap_or(0);
        asts[path]
            .index
            .indexed_lookup_paths
            .iter()
            .filter_map(|current_path| {
                asts[current_path]
                    .index
                    .indexed_nodes
                    .get(&self.slice)
                    .map(|nodes| (nodes, current_path))
            })
            .flat_map(|(nodes, current_path)| {
                nodes.iter().map(move |node| (node, current_path)).rev()
            })
            .find(|(node, current_path)| {
                let node_parent_id = node.parent_ids.last().copied().unwrap_or(0);
                let is_node_root_child = node.parent_ids.len() == 1;
                (node.id < parent_id || &path != current_path)
                    && source_node_config.parents.contains(&node.kind_name)
                    && (is_node_root_child || self.parent_ids.contains(&node_parent_id))
            })
            .map(|(node, _)| node)
    }

    pub(crate) fn nested_sources<'a>(
        &self,
        asts: &'a HashMap<PathBuf, FileAst>,
        path: &Path,
    ) -> Vec<&'a Rc<Self>> {
        self.children()
            .flat_map(|child| child.nested_sources(asts, path))
            .chain(self.source(asts, path))
            .unique_by(|node| node.id)
            .collect_vec()
    }

    pub(crate) fn import_path(&self, current_path: &Path, root_path: &Path) -> PathBuf {
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
            current_path.to_path_buf()
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

    #[allow(clippy::needless_lifetimes)]
    fn scan<'a>(&'a self, f: &mut impl FnMut(&'a Self) -> bool) {
        if f(self) {
            return;
        }
        for child in self.children() {
            child.scan(f);
        }
    }
}

#[derive(Debug)]
pub enum AstNodeInner {
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
