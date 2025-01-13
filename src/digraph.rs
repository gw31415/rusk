use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

/// Node of a tree
pub struct TreeNode<T> {
    /// Inner-Item of the node
    pub item: T,
    /// Children of the node
    pub children: Vec<Rc<TreeNode<T>>>,
}

/// Error of TreeNode
#[derive(Debug, thiserror::Error)]
pub enum TreeNodeCreationError {
    /// Dependency not found
    #[error("Dependency named {0:?} not found")]
    DependencyNotFound(String),
    /// Circular dependency found
    #[error("Circular dependency found around {0:?}")]
    CircularDependency(String),
}

impl<D: DigraphItem> TreeNode<D> {
    /// Create trees from a directed graph.
    pub fn new_vec(
        mut hashmap: HashMap<String, D>,
        targets: impl IntoIterator<Item: AsRef<str>>,
    ) -> Result<Vec<Self>, TreeNodeCreationError> {
        fn convert<D: DigraphItem>(
            base: &mut HashMap<String, D>,
            converted: &mut HashMap<String, (Rc<TreeNode<D>>, HashSet<String>)>,
            item: D,
        ) -> Result<TreeNode<D>, TreeNodeCreationError> {
            let mut children = vec![];
            for dep_name in item.dependencies() {
                let dep_name = dep_name.as_ref();
                let child = if let Some(dep_item) = base.remove(dep_name) {
                    let node = Rc::new(convert(base, converted, dep_item)?);
                    converted.insert(dep_name.to_string(), (node.clone(), Default::default()));
                    node
                } else if let Some((dep_item, depend_labels_all)) = converted.get_mut(dep_name) {
                    let dep_name = dep_name.to_string();
                    if depend_labels_all.contains(&dep_name) {
                        return Err(TreeNodeCreationError::CircularDependency(dep_name));
                    }
                    depend_labels_all.insert(dep_name);
                    dep_item.clone()
                } else {
                    return Err(TreeNodeCreationError::DependencyNotFound(
                        dep_name.to_string(),
                    ));
                };
                children.push(child);
            }
            Ok(TreeNode::<D> { item, children })
        }

        let mut roots = vec![];
        let mut converted = Default::default();
        for label in targets {
            let Some(d) = hashmap.remove(label.as_ref()) else {
                return Err(TreeNodeCreationError::DependencyNotFound(
                    label.as_ref().to_string(),
                ));
            };
            let node = convert(&mut hashmap, &mut converted, d)?;
            roots.push(node);
        }
        Ok(roots)
    }
}

/// Vertex of a directed graph
pub trait DigraphItem {
    /// Get dependencies of the vertex
    fn dependencies(&self) -> impl IntoIterator<Item: AsRef<str>>;
}
