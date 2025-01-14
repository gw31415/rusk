use std::{
    collections::{HashMap, HashSet},
    future::{Future, IntoFuture},
    pin::Pin,
    rc::Rc,
};

use futures::future::try_join_all;

/// Node of a tree
pub struct TreeNode<T> {
    /// Inner-Item of the node
    item: T,
    /// Children of the node
    children: Vec<Rc<TreeNode<T>>>,
}

impl<T, E, I: IntoFuture<Output = Result<T, E>> + 'static> IntoFuture for TreeNode<I> {
    type Output = Result<T, E>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output>>>;
    fn into_future(self) -> Self::IntoFuture {
        let TreeNode { item, mut children } = self;
        Box::pin(async move {
            while !children.is_empty() {
                let mut buf = Vec::new();
                let mut tasks = Vec::new();
                for child in children {
                    match Rc::try_unwrap(child) {
                        Ok(node) => {
                            tasks.push(Self::into_future(node));
                        }
                        Err(rc) => {
                            buf.push(rc);
                        }
                    }
                }
                try_join_all(tasks).await?;
                children = buf;
            }
            item.await
        })
    }
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
            for dep_name in item.dependencies() {
                let dep_name = dep_name.as_ref();
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
    /// Get dependencies of the vertex
    fn dependencies(&self) -> impl IntoIterator<Item: AsRef<str>>;
}
