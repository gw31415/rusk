use std::{
    collections::{HashMap, HashSet},
    ops::{AddAssign, Deref, DerefMut},
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

/// To manage parents of a node. When the manager is dropped, it removes the parent from the set.
struct ParentsManager<'a>(&'a mut HashSet<String>, Option<String>);

impl<'a> From<&'a mut HashSet<String>> for ParentsManager<'a> {
    fn from(val: &'a mut HashSet<String>) -> Self {
        ParentsManager(val, None)
    }
}

impl Deref for ParentsManager<'_> {
    type Target = HashSet<String>;
    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl DerefMut for ParentsManager<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0
    }
}

impl Drop for ParentsManager<'_> {
    fn drop(&mut self) {
        if let Some(name) = self.1.take() {
            self.0.remove(&name);
        }
    }
}

impl AddAssign<String> for ParentsManager<'_> {
    fn add_assign(&mut self, rhs: String) {
        self.0.insert(rhs.clone());
        self.1 = Some(rhs);
    }
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
            parents: &mut HashSet<String>,
        ) -> Result<TreeNode<D>, TreeNodeCreationError> {
            let mut parents: ParentsManager = parents.into();
            parents += name.clone();

            let mut children = vec![];
            for dep_name in raw.children().iter() {
                let dep_name = dep_name.as_ref();
                if parents.contains(dep_name) {
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
                            Rc::new(convert(dep_name.to_string(), dep_item, list, &mut parents)?);
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
                let node = convert(label, raw, &mut hashmap, &mut HashSet::new())?;
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
