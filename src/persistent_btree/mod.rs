mod node;

use node::*;
use std::{fmt::Display, marker::PhantomData, ptr::NonNull};

use crate::persistent_btree::node::Modification;
use crate::persistent_btree::node::NodeSnapshot;
use crate::persistent_btree::node::ReturnPtr;

pub struct PBTree<T>
where
    T: PartialOrd + Clone,
{
    // TODO: we need a way to update root_history if the root has changed
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
            if *root_version <= version {
                root = link.clone()
            } else {
                break;
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

    /// Transplants a node: Switches the node's parent's child from self to target.
    fn transplant(&mut self, old_node: &mut Node<T>, new_node: Link<T>) {
        // Get latest parent link
        let mut parent_link = old_node.with_mods().parent;

        // Updates parent to point to target instead of self
        if let Some(mut parent_ptr) = parent_link {
            // Gets parent and updated parent
            let parent = unsafe { parent_ptr.as_mut() };
            let current_parent = parent.with_mods();

            // Check if we're the parent's left or right child
            let modifier = if current_parent.left.is_some_and(|ptr| ptr == old_node.as_ptr()) {
                Modification::left(new_node, self.version)
            // Else we're the parent's right child
            } else {
                Modification::right(new_node, self.version)
            };

            // We need to check if we generated a new node after
            // add_mod was called to update the parent_link for the target's modification.
            let new_link = parent.add_mod(modifier);
            if new_link.is_some() {
                parent_link = new_link
            }
        } else {
            // TODO: change root
        }

        // Change the target's parent if the target is not None.
        if let Some(mut ptr) = new_node {
            let target_node = unsafe { ptr.as_mut() };
            target_node.add_mod(Modification::parent(parent_link, self.version));
        }
    }

    pub fn remove(&mut self, mut node: &mut Node<T>) {
        self.version += 1;
        let latest_node = node.with_mods();

        if latest_node.left.is_none() {
            self.transplant(node, latest_node.right);
        } else if latest_node.right.is_none() {
            self.transplant(node, latest_node.left);
        } else {
            // we can unwrap because we're guaranteed to have at least a right child.
            let mut successor_ptr = node.with_mods().latest_successor().unwrap();
            let mut successor_link = Some(successor_ptr);
            let successor = unsafe { successor_ptr.as_mut() };
            let is_direct_child = successor.parent.unwrap() == node.as_ptr();

            // Remove successor from its place
            self.transplant(successor, successor.right);

            // If the successor is a direct child to the target node, the transplant might
            // cause unexpected modifications to the target node due to the recursion in add_mod
            if is_direct_child {
                // Updates node
                node = unsafe { successor.parent.unwrap().as_mut() };
            }

            // Put the successor in place of the target node.
            self.transplant(node, successor_link);

            // Override latest_node after the transplants (right could be None)
            let latest_node = node.with_mods();

            // Updates left child
            let new_link = successor.add_mod(Modification::left(latest_node.left, self.version));
            if new_link.is_some() {
                successor_link = new_link
            }
            unsafe {
                let left = latest_node.left.unwrap().as_mut();
                left.add_mod(Modification::parent(successor_link, self.version))
            };

            // Updates right child (checks if right != None)
            let new_link = successor.add_mod(Modification::right(latest_node.right, self.version));
            if new_link.is_some() {
                successor_link = new_link
            }
            if latest_node.right.is_some() {
                unsafe {
                    let right = latest_node.right.unwrap().as_mut();
                    right.add_mod(Modification::parent(successor_link, self.version))
                };
            }
        }
    }
}

// TODO: impl Drop :P
// i <3 memory leaks
