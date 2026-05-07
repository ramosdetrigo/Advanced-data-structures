#![allow(dead_code)]
mod node;

use node::*;
use std::{fmt::Debug, marker::PhantomData, ptr::NonNull};

use crate::persistent_btree::node::Modification;
use crate::persistent_btree::node::ReturnPtr;

#[derive(Debug)]
pub struct PBTree<T>
where
    T: PartialOrd + Clone + Debug,
{
    root_history: Vec<(Link<T>, usize)>,
    version: usize,
    _t: PhantomData<T>,
}

impl<T> PBTree<T>
where
    T: PartialOrd + Clone + Debug,
{
    /// Returns a string listing every node of the tree in crescent order with its depth
    pub fn list(&self, version: usize) -> String {
        let mut stack: Vec<(Node<T>, usize)> = Vec::new();
        let mut depth = 0;
        let mut curr_node = self.root(version);
        let mut result = String::new();

        while curr_node.is_some() || !stack.is_empty() {
            // Go as left as possible
            while let Some(node_ptr) = curr_node {
                let snap = unsafe { node_ptr.as_ref().snapshot(version) };
                curr_node = snap.left;
                stack.push((snap, depth));
                depth += 1;
            }

            // Read current node
            let (snap, node_depth) = stack.pop().unwrap();
            result.push_str(&format!("{:?},{} ", snap.value, node_depth));

            // Read right node
            curr_node = snap.right;
            depth = node_depth + 1
        }

        result
    }

    #[must_use]
    // função gerada pelo chatgpt :p
    // só queria algo pra visualizar melhor a árvore
    pub fn tree_str(&self, version: usize) -> String {
        fn build_tree<T>(
            node: Link<T>,
            version: usize,
            prefix: String,
            is_left: bool,
            result: &mut String,
        ) where
            T: PartialOrd + Clone + Debug,
        {
            if let Some(node_ptr) = node {
                let snap = unsafe { node_ptr.as_ref().snapshot(version) };

                // RIGHT subtree first (goes up)
                build_tree(
                    snap.right,
                    version,
                    format!("{}{}", prefix, if is_left { "│   " } else { "    " }),
                    false,
                    result,
                );

                // CURRENT node
                result.push_str(&format!(
                    "{}{}{:?}\n",
                    prefix,
                    if is_left { "└── " } else { "┌── " },
                    snap.value
                ));

                // LEFT subtree (goes down)
                build_tree(
                    snap.left,
                    version,
                    format!("{}{}", prefix, if is_left { "    " } else { "│   " }),
                    true,
                    result,
                );
            }
        }

        let mut result = String::new();
        build_tree(
            self.root(version),
            version,
            String::new(),
            true,
            &mut result,
        );
        result
    }
}

impl<T> PBTree<T>
where
    T: PartialOrd + Clone + Debug,
{
    #[must_use]
    pub fn new() -> Self {
        Self {
            root_history: Vec::new(),
            version: 0,
            _t: PhantomData,
        }
    }

    #[must_use]
    /// Returns the root of the tree at a certain version
    fn root(&self, version: usize) -> Link<T> {
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

    #[inline(always)]
    #[must_use]
    pub fn version(&self) -> usize {
        self.version
    }

    #[must_use]
    /// Finds the smallest strictly greater than X element in the tree
    pub fn successor(&self, elem: &T, version: usize) -> Option<T> {
        let mut successor: Option<T> = None;
        let mut curr_node = self.root(version);
        while let Some(node) = curr_node {
            let snap = unsafe { node.as_ref().snapshot(version) };
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

    #[must_use]
    /// Finds the node that would be the parent of an element in the tree
    fn find_parent_node(&self, elem: &T, version: usize) -> Link<T> {
        let mut prev_node: Link<T> = None;
        let mut curr_node = self.root(version);

        // Searches the correct place to add the node
        while let Some(node) = curr_node {
            prev_node = curr_node;
            let snap = unsafe { node.as_ref().snapshot(version) };
            if *elem < snap.value {
                curr_node = snap.left
            } else {
                curr_node = snap.right
            }
        }

        prev_node
    }

    /// Finds a node with a value that equals elem at a certain version
    fn find(&self, elem: &T, version: usize) -> Link<T> {
        let mut curr_node = self.root(version);

        while let Some(node) = curr_node {
            let snap = unsafe { node.as_ref().snapshot(version) };
            if *elem == snap.value {
                return curr_node;
            } else if *elem < snap.value {
                curr_node = snap.left
            } else {
                curr_node = snap.right
            }
        }

        None
    }

    /// Adds an element to the tree
    pub fn push(&mut self, elem: T) {
        self.version += 1;
        let parent_link = self.find_parent_node(&elem, self.version);

        let tree_ptr = NonNull::from(&*self);
        if let Some(mut parent) = parent_link {
            // Accesses the parent's data at current version
            let parent = unsafe { parent.as_mut() };
            let parent_snapshot = parent.modded_clone();
            let is_left_child: bool = elem < parent_snapshot.value;

            // Only creates the new node after the check (prevents moving elem)
            let mut new_node = Node::new(elem, tree_ptr);
            // Sets the node's parent without adding to the mod field
            new_node.apply_mod(&Modification::parent(parent_link, self.version));

            // Converts the node into a heap-allocated link
            let link = new_node.into_link();
            // and adds the return ptr to the parent
            parent.add_return_ptr(ReturnPtr::ParentOf(link.unwrap()));

            if is_left_child {
                let _ = parent.add_mod(Modification::left(link, self.version));
            } else {
                let _ = parent.add_mod(Modification::right(link, self.version));
            }
        } else {
            // Edge case: tree is empty
            let new_node = Node::new(elem, tree_ptr).into_link();
            self.root_history.push((new_node, self.version));
        }
    }

    /// Transplants a node: Switches the node's parent's child from self to target.
    /// Returns a new link to the new_node if one was created with add_mod
    #[must_use]
    fn transplant(&mut self, old_node: &mut Node<T>, new_node: Link<T>) -> Link<T> {
        // Get latest parent link
        let mut parent_link = old_node.modded_clone().parent;

        // Updates parent to point to target instead of self
        if let Some(mut parent_ptr) = parent_link {
            // Gets parent and updated parent
            let parent = unsafe { parent_ptr.as_mut() };
            let current_parent = parent.modded_clone();

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
            self.root_history.push((new_node, self.version));
        }

        // Change the target's parent if the target is not None.
        if let Some(mut ptr) = new_node {
            let target_node = unsafe { ptr.as_mut() };
            return target_node.add_mod(Modification::parent(parent_link, self.version));
        }

        None
    }

    // Helper to remove return pointers from the node's neighbours to avoid making it
    // and its children stale nodes through redirection when adding mods.
    // (also to help prevent overflow in the node return_pts vec)
    // Used only in remove(). We can remove the return pointers since this won't be
    // pointing to anything after the removal operation is finished anyways.
    fn remove_ret_pts_to(node: &Node<T>) {
        let node_ptr = node.as_ptr();
        let node = node.modded_clone();
        unsafe {
            if node.parent.is_some() {
                node.parent.unwrap().as_mut().remove_return_ptr(ReturnPtr::ParentOf(node_ptr));
            }
            if node.right.is_some() {
                node.right.unwrap().as_mut().remove_return_ptr(ReturnPtr::RightOf(node_ptr));
            }
            if node.left.is_some() {
                node.left.unwrap().as_mut().remove_return_ptr(ReturnPtr::LeftOf(node_ptr));
            }
        }
    }

    pub fn remove(&mut self, elem: &T) {
        self.version += 1;

        let node_link = self.find(elem, self.version);
        if node_link.is_none() {
            return;
        }
        let mut node = unsafe { node_link.unwrap().as_mut() };

        let latest_node = node.modded_clone();

        if latest_node.left.is_none() {
            PBTree::remove_ret_pts_to(node);
            let _ = self.transplant(node, latest_node.right);
        } else if latest_node.right.is_none() {
            PBTree::remove_ret_pts_to(node);
            let _ = self.transplant(node, latest_node.left);
        } else {
            let mut successor_link = node.modded_clone().latest_successor();
            // we can unwrap because we're guaranteed to have at least a right child.
            let mut successor = unsafe { successor_link.unwrap().as_mut() };
            let successor_latest = successor.modded_clone();
            let is_direct_child = successor_latest.parent.unwrap() == node.as_ptr();

            // Part 1: switch the successor with its right child.
            unsafe {
                // remove return pointers from the successor's parent to avoid making successor
                // a stale node through redirection when adding mods to the parent.
                // we can remove the return pointer since this won't be pointing to the parent after
                // the removal operation is finished anyways.
                successor
                    .parent
                    .unwrap()
                    .as_mut()
                    .remove_return_ptr(ReturnPtr::ParentOf(successor_link.unwrap()));
            };

            // Then remove successor from its place
            let _ = self.transplant(successor, successor_latest.right);
            // If the successor is a direct child to the target node, the transplant might
            // cause the target node to be stale due to the recursion in add_mod
            if is_direct_child {
                // Updates node
                node = unsafe { successor_latest.parent.unwrap().as_mut() };
            }

            // Part 2: put the successor in place of the target node.
            PBTree::remove_ret_pts_to(node);
            let new_link = self.transplant(node, successor_link);
            if new_link.is_some() {
                successor_link = new_link;
                successor = unsafe { successor_link.unwrap().as_mut() };
            }
            // Override latest_node after the transplants
            // (right could now be None if the successor was a direct child)
            let latest_node = node.modded_clone();

            // Updates left child
            let new_link = successor.add_mod(Modification::left(latest_node.left, self.version));
            if new_link.is_some() {
                successor_link = new_link;
                successor = unsafe { successor_link.unwrap().as_mut() };
            }

            let left = unsafe { latest_node.left.unwrap().as_mut() };
            let new_left_link = left.add_mod(Modification::parent(successor_link, self.version));
            if new_left_link.is_some() {
                // Updates successor just in case it was made into a new node through recursion
                successor_link = unsafe { new_left_link.unwrap().as_mut() }.parent;
                successor = unsafe { successor_link.unwrap().as_mut() }
            }

            // Updates right child (checks if right != None)
            let new_link = successor.add_mod(Modification::right(latest_node.right, self.version));
            if new_link.is_some() {
                successor_link = new_link
                // no need to update successor anymore
            }
            if latest_node.right.is_some() {
                unsafe {
                    let right = latest_node.right.unwrap().as_mut();
                    let _ = right.add_mod(Modification::parent(successor_link, self.version));
                }
            }
        }
    }
}

#[cfg(test)]
mod pbtree_test {
    use super::PBTree;
    #[test]
    fn basic() {
        let mut tree: PBTree<i32> = PBTree::new();
        tree.push(1);
        tree.push(3); //1
        tree.push(4);
        tree.push(5);
        tree.push(6);
        tree.remove(&3); //2
        tree.remove(&4); //3
        tree.remove(&5); //4
        tree.remove(&6); //5
        tree.push(2); //6
        tree.push(3); //1
        tree.remove(&3); //2
        tree.push(3); //3
        tree.remove(&3); //4
        tree.push(3); //5
        tree.remove(&3); //6
        tree.push(3); //7

        for i in 1..=tree.version() {
            println!("====== TREE AT VERSION {i} ======");
            println!("{}", tree.list(i));
            println!("{}", tree.tree_str(i));
        }
    }
}

// TODO: impl Drop :P
// i <3 memory leaks
