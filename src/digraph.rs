use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
    marker::PhantomData,
    ops::{AddAssign, Deref, DerefMut},
    rc::Rc,
};

/// Node of a tree
pub struct TreeNode<K: Hash + Eq + Clone, T> {
    _key: PhantomData<fn() -> K>,
    /// Inner-Item of the node
    pub item: T,
    /// Children of the node
    pub children: Vec<Rc<TreeNode<K, T>>>,
}

/// Error of TreeNode
#[derive(Debug, thiserror::Error)]
pub enum TreeNodeCreationError<K: Hash + Eq + Clone> {
    /// Item not found
    #[error("Item named {0:?} not found")]
    ItemNotFound(K),
    /// Circular dependency found
    #[error("Circular dependency found around {0:?}")]
    CircularDependency(K),
}

/// To manage parents of a node. When the manager is dropped, it removes the parent from the set.
struct ParentsManager<'a, K: Hash + Eq + Clone>(&'a mut HashSet<K>, Option<K>);

impl<'a, K: Hash + Eq + Clone> From<&'a mut HashSet<K>> for ParentsManager<'a, K> {
    fn from(val: &'a mut HashSet<K>) -> Self {
        ParentsManager(val, None)
    }
}

impl<'a, K: Hash + Eq + Clone> Deref for ParentsManager<'a, K> {
    type Target = HashSet<K>;
    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<'a, K: Hash + Eq + Clone> DerefMut for ParentsManager<'a, K> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0
    }
}

impl<'a, K: Hash + Eq + Clone> Drop for ParentsManager<'a, K> {
    fn drop(&mut self) {
        if let Some(name) = self.1.take() {
            self.0.remove(&name);
        }
    }
}

impl<'a, K: Hash + Eq + Clone> AddAssign<K> for ParentsManager<'a, K> {
    fn add_assign(&mut self, rhs: K) {
        self.0.insert(rhs.clone());
        self.1 = Some(rhs);
    }
}

impl<K: Hash + Eq + Clone, D: DigraphItem<K>> TreeNode<K, D> {
    /// Create trees from a directed graph.
    pub fn new_vec(
        hashmap: HashMap<K, D>,
        targets: impl IntoIterator<Item = K>,
    ) -> Result<Vec<Self>, TreeNodeCreationError<K>> {
        enum RawOrNode<K: Hash + Eq + Clone, D: DigraphItem<K>> {
            Raw(D),
            Node(Rc<TreeNode<K, D>>),
        }
        fn convert<K: Hash + Eq + Clone, D: DigraphItem<K>>(
            name: K,
            raw: D,
            list: &mut HashMap<K, RawOrNode<K, D>>,
            parents: &mut HashSet<K>,
        ) -> Result<TreeNode<K, D>, TreeNodeCreationError<K>> {
            let mut parents: ParentsManager<K> = parents.into();
            parents += name.clone();

            let mut children = vec![];
            for dep_name in raw.children().iter() {
                if parents.contains(dep_name) {
                    return Err(TreeNodeCreationError::CircularDependency(dep_name.clone()));
                }

                let Some(item) = list.remove(dep_name) else {
                    return Err(TreeNodeCreationError::ItemNotFound(dep_name.clone()));
                };
                match item {
                    RawOrNode::Raw(dep_item) => {
                        let node =
                            Rc::new(convert(dep_name.clone(), dep_item, list, &mut parents)?);
                        list.insert(dep_name.clone(), RawOrNode::Node(node.clone()));
                        children.push(node);
                    }
                    RawOrNode::Node(dep_node) => {
                        list.insert(dep_name.clone(), RawOrNode::Node(dep_node.clone()));
                        children.push(dep_node);
                    }
                }
            }
            Ok(TreeNode::<K, D> {
                _key: PhantomData,
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
pub trait DigraphItem<K: Hash + Eq + Clone> {
    /// Get children of the vertex
    fn children(&self) -> impl Deref<Target = [K]>;
}
