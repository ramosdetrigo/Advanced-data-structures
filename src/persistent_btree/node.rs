use std::fmt::Debug;

use super::tools::*;
use super::*;

pub type NodePtr<K, V> = NonNull<RawNode<K, V>>;
pub type Link<K, V> = Option<NodePtr<K, V>>;

const P: usize = 3; // Número de ponteiros em um nó

#[derive(Clone, Debug)]
pub struct NodeHandle<K, V>
where
    K: PartialOrd + Clone + Debug,
    V: Clone + Debug
{
    node: NodePtr<K, V>,
}

#[derive(Clone, Debug)]
pub struct NodeSnapshot<K, V>
where
    K: PartialOrd + Clone + Debug,
    V: Clone + Debug
{
    node: NodePtr<K, V>,
    version: usize
}

#[derive(Clone, Debug)]
pub(super) struct RawNode<K, V>
where
    V: Clone + Debug,
    K: PartialOrd + Clone + Debug,
{
    key: K,
    value: V,
    parent: Link<K, V>,
    left: Link<K, V>,
    right: Link<K, V>,
    // NOTE: Since we're using a vec, the P and 2P restrictions need to be enforced via code!
    return_pts: Vec<ReturnPtr<K, V>>, // P = 3
    mods: Vec<Modification<K, V>>,    // 2P = 6
    handle: NonNull<NodeHandle<K, V>>
}

impl<K, V> RawNode<K, V>
where
    K: PartialOrd + Clone + Debug,
    V: Clone + Debug
{
    // Node constructor
    #[must_use]
    #[inline]
    pub fn new(key: K, value: V, parent: Link<K, V>, left: Link<K, V>, right: Link<K, V>) -> Self {
        Self {
            key,
            value,
            parent: None,
            left: None,
            right: None,
            return_pts: Vec::<ReturnPtr<K, V>>::with_capacity(P),
            mods: Vec::with_capacity(P * 2),
            handle: NonNull::dangling()
        }
    }

    pub fn add_mod(&mut self, modification: Modification<K, V>) {
        if self.mods.len() >= 6 {
            let mut new_node = self.modded_clone();
            unsafe { self.handle.as_mut().node = new_node.as_ptr() };
            new_node.add_mod(modification);
            return;
        }
    }

    fn handle_mod(&mut self, modification: Modification<K, V>) {

    }

    fn apply_mod(&mut self, modification: &Modification<K, V>) {

    }

    pub fn modded_clone(&self) -> RawNode<K, V> {
        let clone = Self::new(self.key.clone(), self.value.clone(), self.parent, self.left, self.right);
        clone
    }

    pub fn as_ptr(&self) -> NodePtr<K, V> {
        NonNull::from(self)
    }
}
