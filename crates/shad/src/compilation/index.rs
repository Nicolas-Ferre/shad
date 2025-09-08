use crate::compilation::node::Node;
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
        let criteria = node.source_search_criteria();
        let parent_id = node.parent_ids.last().copied().unwrap_or(0);
        for current_path in self.lookup_paths.get(&node.path)? {
            if let Some(nodes) = self.nodes.get(current_path)?.get(key) {
                for source_node in nodes.iter().rev() {
                    let node_parent_id = source_node.parent_ids.last().copied().unwrap_or(0);
                    let is_node_root_child = source_node.parent_ids.len() == 2;
                    let is_matching = criteria.iter().any(|criteria| {
                        let has_node_min_parent_count =
                            criteria
                                .common_parent_count
                                .is_some_and(|common_parent_count| {
                                    source_node.parent_ids.len() >= common_parent_count
                                        && node.parent_ids.len() >= common_parent_count
                                        && source_node.parent_ids[..common_parent_count]
                                            == node.parent_ids[..common_parent_count]
                                });
                        (criteria.can_be_after
                            || source_node.id < parent_id
                            || &node.path != current_path)
                            && source_node.node_type_id() == (criteria.node_type)()
                            && (is_node_root_child
                                || has_node_min_parent_count
                                || node.parent_ids.contains(&node_parent_id))
                    });
                    if is_matching {
                        return Some(&**source_node);
                    }
                }
            }
        }
        None
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
        Self::find_inner_lookup_paths(path, root_path, roots, &mut paths, &mut unique_paths);
        if !unique_paths.contains(Path::new(PRELUDE_PATH)) {
            paths.push(PRELUDE_PATH.into());
        }
        paths
    }

    fn find_inner_lookup_paths(
        path: &Path,
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
            if !unique_paths.contains(&import_path) {
                unique_paths.insert(import_path.clone());
                registered_paths.push(import_path.clone());
                Self::find_inner_lookup_paths(
                    &import_path,
                    root_path,
                    roots,
                    registered_paths,
                    unique_paths,
                );
            }
        }
    }
}
