use crate::compilation::node::{Node, NodeConfig};
use crate::compilation::PRELUDE_PATH;
use crate::language::items::Root;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::rc::Rc;

#[derive(Default, Debug)]
pub(crate) struct NodeIndex {
    nodes: HashMap<PathBuf, HashMap<String, Vec<Rc<dyn Node>>>>,
    lookup_paths: HashMap<PathBuf, Vec<PathBuf>>,
}

impl NodeIndex {
    pub(crate) fn new(roots: &HashMap<PathBuf, Root>, root_path: &Path) -> Self {
        let mut self_ = Self::default();
        for root in roots.values() {
            root.index(&mut self_);
            self_.lookup_paths.insert(
                root.path.clone(),
                Self::find_lookup_paths(root, roots, root_path),
            );
        }
        self_
    }

    pub(crate) fn register<T: Node>(&mut self, key: String, node: &Rc<T>) {
        self.nodes
            .entry(node.path.clone())
            .or_default()
            .entry(key)
            .or_default()
            .push(node.clone());
    }

    pub(crate) fn search(&self, node: &impl Node, key: &str) -> Option<&dyn Node> {
        for current_path in self.lookup_paths.get(&node.path)? {
            if let Some(nodes) = self
                .nodes
                .get(current_path)
                .and_then(|paths| paths.get(key))
            {
                let mut found_source: Option<&dyn Node> = None;
                for source in nodes.iter().rev() {
                    if found_source
                        .is_none_or(|found| found.parent_ids.len() < source.parent_ids.len())
                        && Self::is_source_in_scope(node, source, current_path)
                    {
                        found_source = Some(&**source);
                    }
                }
                if found_source.is_some() {
                    return found_source;
                }
            }
        }
        None
    }

    fn is_source_in_scope(node: &impl Node, source: &Rc<dyn Node>, current_path: &PathBuf) -> bool {
        let node_parent_id = node.parent_ids.last().copied().unwrap_or(0);
        let source_parent_id = source.parent_ids.last().copied().unwrap_or(0);
        let is_source_root = source.parent_ids.len() == 2;
        node.source_search_criteria().iter().any(|criteria| {
            let has_source_min_parent_count =
                criteria
                    .common_parent_count
                    .is_some_and(|common_parent_count| {
                        source.parent_ids.len() >= common_parent_count
                            && node.parent_ids.len() >= common_parent_count
                            && source.parent_ids[..common_parent_count]
                                == node.parent_ids[..common_parent_count]
                    });
            (&node.path == current_path || source.is_public())
                && (criteria.can_be_after
                    || source.id < node_parent_id
                    || &node.path != current_path)
                && source.node_type_id() == (criteria.node_type)()
                && (is_source_root
                    || has_source_min_parent_count
                    || node.parent_ids.contains(&source_parent_id))
        })
    }

    fn find_lookup_paths(
        root: &Root,
        roots: &HashMap<PathBuf, Root>,
        root_path: &Path,
    ) -> Vec<PathBuf> {
        let path = &root.path;
        let mut paths = vec![path.clone()];
        let mut unique_paths = HashSet::new();
        unique_paths.insert(path.clone());
        Self::find_inner_lookup_paths(path, true, root_path, roots, &mut paths, &mut unique_paths);
        if !unique_paths.contains(Path::new(PRELUDE_PATH)) {
            paths.push(PRELUDE_PATH.into());
        }
        paths
    }

    fn find_inner_lookup_paths(
        path: &Path,
        is_current_path: bool,
        root_path: &Path,
        roots: &HashMap<PathBuf, Root>,
        registered_paths: &mut Vec<PathBuf>,
        unique_paths: &mut HashSet<PathBuf>,
    ) {
        for import in roots
            .get(path)
            .iter()
            .flat_map(|root| root.items.iter().filter_map(|item| item.as_import()))
            .rev()
        {
            let import_path = import.import_path(root_path);
            if (is_current_path || import.is_public()) && !unique_paths.contains(&import_path) {
                unique_paths.insert(import_path.clone());
                registered_paths.push(import_path.clone());
                Self::find_inner_lookup_paths(
                    &import_path,
                    false,
                    root_path,
                    roots,
                    registered_paths,
                    unique_paths,
                );
            }
        }
    }
}
