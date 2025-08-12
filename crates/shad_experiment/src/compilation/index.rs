use crate::compilation::ast::{AstNode, AstNodeInner};
use crate::config::Config;
use crate::FileAst;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::rc::Rc;

#[derive(Debug)]
pub(crate) struct AstNodeIndex {
    pub(crate) indexed_nodes: HashMap<String, Vec<Rc<AstNode>>>,
    pub(crate) indexed_lookup_paths: Vec<PathBuf>,
}

impl AstNodeIndex {
    pub(crate) fn new(ast: &Rc<AstNode>) -> Self {
        let mut indexed_nodes = HashMap::new();
        fill_index(ast, &mut indexed_nodes);
        Self {
            indexed_nodes,
            indexed_lookup_paths: vec![],
        }
    }

    #[allow(clippy::needless_collect)]
    pub(crate) fn generate_lookup_paths(
        config: &Config,
        asts: &mut HashMap<PathBuf, FileAst>,
        root_path: &Path,
    ) {
        for path in asts.keys().cloned().collect::<Vec<_>>() {
            let mut paths = vec![path.clone()];
            let mut unique_paths = HashSet::new();
            unique_paths.insert(path.clone());
            Self::register_import_paths(
                config,
                asts,
                root_path,
                &path,
                &mut paths,
                &mut unique_paths,
            );
            asts.get_mut(&path)
                .expect("internal error: missing AST")
                .index
                .indexed_lookup_paths = paths;
        }
    }

    fn register_import_paths(
        config: &Config,
        asts: &HashMap<PathBuf, FileAst>,
        root_path: &Path,
        path: &Path,
        registered_paths: &mut Vec<PathBuf>,
        unique_paths: &mut HashSet<PathBuf>,
    ) {
        for import in asts[path]
            .index
            .indexed_nodes
            .get(&config.import_index_key)
            .iter()
            .flat_map(|nodes| nodes.iter())
            .rev()
        {
            let import_path = import.import_path(path, root_path);
            if !unique_paths.contains(&import_path) {
                unique_paths.insert(import_path.clone());
                registered_paths.push(import_path.clone());
                Self::register_import_paths(
                    config,
                    asts,
                    root_path,
                    &import_path,
                    registered_paths,
                    unique_paths,
                );
            }
        }
    }
}

fn fill_index(node: &Rc<AstNode>, index: &mut HashMap<String, Vec<Rc<AstNode>>>) {
    if let Some(index_key) = &node.kind_config.index_key {
        let key = if let Some(child) = &index_key.child {
            node.child(child).slice.clone()
        } else if let Some(string) = &index_key.string {
            string.clone()
        } else {
            unreachable!("index key config should be valid");
        };
        index.entry(key).or_default().push(node.clone());
    }
    match &node.children {
        AstNodeInner::Sequence(children) => {
            for child in children.values() {
                fill_index(child, index);
            }
        }
        AstNodeInner::Repeated(children) => {
            for child in children {
                fill_index(child, index);
            }
        }
        AstNodeInner::Terminal => {}
    }
}
