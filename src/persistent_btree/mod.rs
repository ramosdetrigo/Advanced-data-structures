mod node;

use node::Link;
use node::Node;
use std::{fmt::Display, marker::PhantomData, ptr::NonNull};

use crate::persistent_btree::node::Modification;
use crate::persistent_btree::node::NodeSnapshot;
use crate::persistent_btree::node::ReturnPtr;

pub struct PBTree<T>
where
    T: PartialOrd + Clone,
{
    root_history: Vec<(Link<T>, usize)>,
    version: usize,
    _t: PhantomData<T>,
}

impl<T> PBTree<T>
where
    T: PartialOrd + Clone + Display,
{
    pub fn list(&self, version: usize) -> String {
        let mut stack: Vec<(NodeSnapshot<T>, usize)> = Vec::new();
        let mut depth = 0;
        let mut curr_node = self.root(version);
        let mut result = String::new();

        while curr_node.is_some() || !stack.is_empty() {
            // Go as left as possible
            while let Some(node_ptr) = curr_node {
                let snap = unsafe { node_ptr.as_ref().at_version(version) };
                curr_node = snap.left;
                stack.push((snap, depth));
                depth += 1;
            }

            // Read current node
            let (snap, node_depth) = stack.pop().unwrap();
            result.push_str(&format!("(v: {}, d: {})\n", snap.value, node_depth));

            // Read right node
            curr_node = snap.right;
            depth = node_depth + 1
        }

        result
    }
}

impl<T> PBTree<T>
where
    T: PartialOrd + Clone,
{
    pub fn new() -> Self {
        Self {
            root_history: Vec::new(),
            version: 0,
            _t: PhantomData,
        }
    }

    pub fn root(&self, version: usize) -> Link<T> {
        let mut root: Link<T> = None;
        for (link, root_version) in &self.root_history {
            if version <= *root_version {
                root = link.clone()
            }
        }
        root
    }

    pub fn version(&self) -> usize {
        self.version
    }

    /// Finds the smallest strictly greater than X element in the tree
    pub fn successor(&self, elem: &T, version: usize) -> Option<T> {
        let mut successor: Option<T> = None;
        let mut curr_node = self.root(version);
        while let Some(node) = curr_node {
            let snap = unsafe { node.as_ref().at_version(version) };
            // If the value of the node is greater than the element,
            // update successor & search to the left
            if snap.value > *elem {
                successor = Some(snap.value);
                curr_node = snap.left;
            // Else: search to the right
            } else {
                curr_node = snap.right;
            }
        }
        successor
    }

    /// Finds the node that would be the parent of an element in the tree
    fn find_parent_node(&self, elem: &T, version: usize) -> Link<T> {
        let mut prev_node: Link<T> = None;
        let mut curr_node = self.root(version);

        // Searches the correct place to add the node
        while let Some(node) = curr_node {
            prev_node = curr_node;
            let snap = unsafe { node.as_ref().at_version(version) };
            if *elem < snap.value {
                curr_node = snap.left
            } else {
                curr_node = snap.right
            }
        }

        prev_node
    }

    pub fn push(&mut self, elem: T) {
        self.version += 1;
        let parent_link = self.find_parent_node(&elem, self.version);

        if let Some(mut parent) = parent_link {
            // Accesses the parent's data at current version
            let parent = unsafe { parent.as_mut() };
            let parent_snapshot = parent.at_version(self.version);
            let is_left_child: bool = elem < parent_snapshot.value;

            // Only creates the new node after the check (prevents moving elem)
            let mut new_node = Node::new(elem);
            // Sets the node's parent without adding to the mod field
            new_node.apply_mod(&Modification::parent(parent_link, self.version));

            // Converts the node into a heap-allocated link
            let link = new_node.into_link();
            // and adds the return ptr to the parent
            parent.add_return_ptr(ReturnPtr::ParentOf(link.unwrap()));

            if is_left_child {
                parent.add_mod(Modification::left(link, self.version));
            } else {
                parent.add_mod(Modification::right(link, self.version));
            }
        } else {
            // Edge case: tree is empty
            let new_node = Node::new(elem).into_link();
            self.root_history.push((new_node, self.version));
        }
    }
}

// TODO: impl Drop :P
// i <3 memory leaks
