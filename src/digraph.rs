use std::{
    collections::{HashMap, HashSet},
    ops::Deref,
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
        targets: impl IntoIterator<Item = String>,
    ) -> Result<Vec<Self>, TreeNodeCreationError> {
        fn convert<D: DigraphItem>(
            base: &mut HashMap<String, D>,
            converted: &mut HashMap<String, (Rc<TreeNode<D>>, HashSet<String>)>,
            label: String,
        ) -> Result<TreeNode<D>, TreeNodeCreationError> {
            let Some(item) = base.remove(&label) else {
                return Err(TreeNodeCreationError::DependencyNotFound(label));
            };
            let mut children = vec![];
            for dep_name in item.children().iter() {
                let dep_name = dep_name.deref();
                let child = if base.contains_key(dep_name) {
                    let node = Rc::new(convert(base, converted, dep_name.to_string())?);
                    converted.insert(dep_name.to_string(), (node.clone(), Default::default()));
                    node
                } else if let Some((dep_item, depend_labels_all)) = converted.get_mut(dep_name) {
                    if depend_labels_all.contains(&label) {
                        return Err(TreeNodeCreationError::CircularDependency(label));
                    }
                    depend_labels_all.insert(label.clone());
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
            let node = convert(&mut hashmap, &mut converted, label)?;
            roots.push(node);
        }
        Ok(roots)
    }
}

/// Vertex of a directed graph
pub trait DigraphItem {
    /// Get children of the vertex
    fn children(&self) -> impl Deref<Target = [impl Deref<Target = str>]>;
}
