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
    /// Item not found
    #[error("Item named {0:?} not found")]
    ItemNotFound(String),
    /// Circular dependency found
    #[error("Circular dependency found around {0:?}")]
    CircularDependency(String),
}

impl<D: DigraphItem> TreeNode<D> {
    /// Create trees from a directed graph.
    pub fn new_vec(
        hashmap: HashMap<String, D>,
        targets: impl IntoIterator<Item = String>,
    ) -> Result<Vec<Self>, TreeNodeCreationError> {
        enum RawOrNode<D> {
            Raw(D),
            Node(Rc<TreeNode<D>>),
        }
        fn convert<D: DigraphItem>(
            name: String,
            raw: D,
            list: &mut HashMap<String, RawOrNode<D>>,
            parents: &HashSet<&str>,
        ) -> Result<TreeNode<D>, TreeNodeCreationError> {
            let parents = {
                let mut parents = parents.clone();
                parents.insert(&name);
                parents
            };

            let mut children = vec![];
            for dep_name in raw.children().iter() {
                let dep_name = dep_name.as_ref();
                if parents.contains(dep_name) || dep_name == name {
                    return Err(TreeNodeCreationError::CircularDependency(
                        dep_name.to_string(),
                    ));
                }

                let Some(item) = list.remove(dep_name) else {
                    return Err(TreeNodeCreationError::ItemNotFound(dep_name.to_string()));
                };
                match item {
                    RawOrNode::Raw(dep_item) => {
                        let node =
                            Rc::new(convert(dep_name.to_string(), dep_item, list, &parents)?);
                        list.insert(dep_name.to_string(), RawOrNode::Node(node.clone()));
                        children.push(node);
                    }
                    RawOrNode::Node(dep_node) => {
                        list.insert(dep_name.to_string(), RawOrNode::Node(dep_node.clone()));
                        children.push(dep_node);
                    }
                }
            }
            Ok(TreeNode::<D> {
                item: raw,
                children,
            })
        }

        let mut roots = vec![];
        let mut hashmap = hashmap
            .into_iter()
            .map(|(k, v)| (k, RawOrNode::Raw(v)))
            .collect::<HashMap<_, _>>();
        for label in targets {
            let Some(item) = hashmap.remove(&label) else {
                return Err(TreeNodeCreationError::ItemNotFound(label));
            };
            if let RawOrNode::Raw(raw) = item {
                let node = convert(label, raw, &mut hashmap, &Default::default())?;
                roots.push(node);
            }
        }
        Ok(roots)
    }
}

/// Vertex of a directed graph
pub trait DigraphItem {
    /// Get children of the vertex
    fn children(&self) -> impl Deref<Target = [impl AsRef<str>]>;
}
