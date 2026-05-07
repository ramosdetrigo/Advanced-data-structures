use std::fmt::Debug;

use super::*;

pub type NodePtr<T> = NonNull<Node<T>>;
pub type Link<T> = Option<NodePtr<T>>;

const P: usize = 3; // Número de ponteiros em um nó
#[derive(Clone, Debug)]
pub struct Node<T>
where
    T: PartialOrd + Clone + Debug,
{
    pub value: T,
    pub parent: Link<T>,
    pub left: Link<T>,
    pub right: Link<T>,
    pub tree_ptr: NonNull<PBTree<T>>,
    // NOTE: Since we're using a vec, the P and 2P restrictions need to be enforced via code!
    return_pts: Vec<ReturnPtr<T>>, // P = 3
    mods: Vec<Modification<T>>,    // 2P = 6
}

#[derive(Clone, Debug)]
pub struct Modification<T>
where
    T: PartialOrd + Clone + Debug,
{
    version: usize,
    modification: ModTarget<T>,
}

impl<T> Modification<T>
where
    T: PartialOrd + Clone + Debug,
{
    #[inline]
    #[must_use]
    pub fn parent(link: Link<T>, version: usize) -> Self {
        Self {
            version,
            modification: ModTarget::Parent(link),
        }
    }

    #[inline]
    #[must_use]
    pub fn left(link: Link<T>, version: usize) -> Self {
        Self {
            version,
            modification: ModTarget::Left(link),
        }
    }

    #[inline]
    #[must_use]
    pub fn right(link: Link<T>, version: usize) -> Self {
        Self {
            version,
            modification: ModTarget::Right(link),
        }
    }

    #[inline]
    #[must_use]
    #[allow(dead_code)]
    pub fn value(value: T, version: usize) -> Self {
        Self {
            version,
            modification: ModTarget::Value(value),
        }
    }
}

/// Represents a modification to a specific field of the node
#[derive(Clone, Debug)]
pub enum ModTarget<T>
where
    T: PartialOrd + Clone + Debug,
{
    Parent(Link<T>),
    Left(Link<T>),
    Right(Link<T>),
    #[allow(dead_code)]
    Value(T),
}

/// A return pointer to another node. \
/// This reads as "this node is Left/Right/Parent of another node": \
/// `ReturnPtr::LeftOf(Y)` inside `X.return_pts` would mean `Y.left = X`
#[derive(Clone, PartialEq, Debug)]
pub enum ReturnPtr<T>
where
    T: PartialOrd + Clone + Debug,
{
    ParentOf(NodePtr<T>),
    LeftOf(NodePtr<T>),
    RightOf(NodePtr<T>),
}

impl<T> ReturnPtr<T>
where
    T: PartialOrd + Clone + Debug,
{
    /// Adds a modifier to the return_ptr's node to point to the new node
    pub fn redirect_node(&mut self, version: usize, new_target: Link<T>) {
        let (return_ptr, modification) = match self {
            ReturnPtr::LeftOf(ptr) => (ptr, ModTarget::Left(new_target)),
            ReturnPtr::RightOf(ptr) => (ptr, ModTarget::Right(new_target)),
            ReturnPtr::ParentOf(ptr) => (ptr, ModTarget::Parent(new_target)),
        };

        unsafe {
            // We don't need to update return_ptr since add_mod_full should already do that
            // if a new node is created.
            let _ = return_ptr.as_mut().add_mod(Modification {
                version,
                modification,
            });
        };
    }

    #[inline]
    #[must_use]
    /// "Unwraps" the ReturnPtr
    pub fn ptr(&self) -> NodePtr<T> {
        match self {
            ReturnPtr::LeftOf(ptr) => *ptr,
            ReturnPtr::RightOf(ptr) => *ptr,
            ReturnPtr::ParentOf(ptr) => *ptr,
        }
    }

    #[inline]
    /// Sets the ReturnPtr pointer into a new one
    pub fn set_ptr(&mut self, new_ptr: NodePtr<T>) {
        match self {
            ReturnPtr::LeftOf(ptr) => *ptr = new_ptr,
            ReturnPtr::RightOf(ptr) => *ptr = new_ptr,
            ReturnPtr::ParentOf(ptr) => *ptr = new_ptr,
        };
    }
}

impl<T> Node<T>
where
    T: PartialOrd + Clone + Debug,
{
    // Node constructor
    #[must_use]
    #[inline]
    pub fn new(value: T, tree_ptr: NonNull<PBTree<T>>) -> Self {
        Self {
            value,
            tree_ptr,
            parent: None,
            left: None,
            right: None,
            return_pts: Vec::<ReturnPtr<T>>::with_capacity(P),
            mods: Vec::with_capacity(P * 2),
        }
    }

    /// Adds a modifier to the node, handling return pointers and the case
    /// where the mods field is full. \
    /// Returns `Some(NodePtr<T>)` if the mod limit has been reached
    /// and a new node has been created. `None` otherwise.
    /// Marked as must_use to help prevent stale node bugs.
    /// Use as `let _ = [...]` if new links don't matter.
    #[must_use]
    pub fn add_mod(&mut self, modifier: Modification<T>) -> Link<T> {
        // Constraint: the mods field max size is P * 2
        if self.mods.len() == P * 2 {
            return Some(self.add_mod_full(modifier));
        }

        // Case 1: value update
        if let ModTarget::Value(_) = modifier.modification {
            self.mods.push(modifier);
        // Case 2: pointer update
        } else {
            let current = self.modded_clone();

            // old_ptr: Ponteiro pro nó antigo do campo modificado
            // ret_ptr: Padrão do ponteiro de retorno esperado
            // new_ptr: Novo ponteiro do campo modificado
            let (old_ptr, ret_ptr, new_ptr) = match modifier.modification {
                ModTarget::Right(ptr) => (current.right, ReturnPtr::RightOf(self.as_ptr()), ptr),
                ModTarget::Left(ptr) => (current.left, ReturnPtr::LeftOf(self.as_ptr()), ptr),
                ModTarget::Parent(ptr) => (current.parent, ReturnPtr::ParentOf(self.as_ptr()), ptr),
                _ => unreachable!(),
            };

            // Adiciona o mod
            self.mods.push(modifier);

            // Remove o nó atual dos ponteiros de retorno do nó antigo, se necessário
            if let Some(mut node) = old_ptr {
                let node = unsafe { node.as_mut() };
                node.remove_return_ptr(ret_ptr.clone());
            }

            // Adiciona o nó atual nos ponteiros de retorno do novo nó
            if let Some(mut node) = new_ptr {
                let node = unsafe { node.as_mut() };
                node.add_return_ptr(ret_ptr);
            }
        }

        None
    }

    /// Handles the case for add_mod() where the mods field is full.
    #[must_use]
    fn add_mod_full(&mut self, modifier: Modification<T>) -> NodePtr<T> {
        // Create node copy with empty mods field
        let new_node = self.modded_clone();

        // Then convert new_node into a heap-allocated link
        let new_node = new_node.into_link();
        let new_node_ptr = new_node.unwrap();

        // And update nodes that pointed to the old node to point to the new one
        // This will also add the return pointers to the new node.
        let self_ptr = self.as_ptr();
        for return_ptr in &mut self.return_pts {
            // Redirects the target's return pointers to the new node
            let target = unsafe { return_ptr.ptr().as_mut() };
            target.return_pts.iter_mut().for_each(|ret_ptr| {
                if ret_ptr.ptr() == self_ptr {
                    ret_ptr.set_ptr(new_node_ptr)
                }
            });

            // Redirect the non-return pointers to the new node
            // (also adds the return_ptr to the new node since that's how add_mod works.)
            return_ptr.redirect_node(modifier.version, new_node);
        }
        // nothing points to the old node (self) anymore.
        self.return_pts.clear();

        // Finally, adds the modifier to the new node.
        let node_ref = unsafe { new_node.unwrap().as_mut() };
        let _ = node_ref.add_mod(modifier);

        // Remove the mod from the list and immediately apply it:
        // The node when accessed at the current version should already have the first mod applied
        let modifier = node_ref.mods.pop().unwrap();
        let version = modifier.version;
        node_ref.apply_mod(&modifier);

        // Updates the tree's root if it changed
        if self.parent.is_none() {
            let tree = unsafe { self.tree_ptr.as_mut() };
            tree.root_history.push((new_node, version));
        }

        // Returns the pointer to the new node
        new_node.unwrap()
    }

    #[must_use]
    // Returns the latest successor of the node. Does this by getting the
    // leftmost node of its right child's subtree.
    pub fn latest_successor(&self) -> Link<T> {
        let mut curr = self.right;

        while let Some(node) = curr {
            let latest = unsafe { node.as_ref().modded_clone() };

            match latest.left {
                // If there is a node to the left, continue
                Some(left) => curr = Some(left),
                // Else: current node is the successor
                None => return Some(node),
            }
        }

        None
    }

    #[inline]
    pub fn add_return_ptr(&mut self, ret_ptr: ReturnPtr<T>) {
        assert!(
            self.return_pts.len() < P,
            "Return pointer vec length exceeded {}!",
            P
        );
        self.return_pts.push(ret_ptr);
    }

    #[inline]
    pub fn remove_return_ptr(&mut self, ret_ptr: ReturnPtr<T>) {
        self.return_pts.retain(|ptr| *ptr != ret_ptr);
    }

    // #[inline]
    // pub fn remove_return_ptr_generic(&mut self, node_ptr: NodePtr<T>) {
    //     self.return_pts.retain(|ret_ptr| ret_ptr.ptr() != node_ptr);
    // }

    /// Returns a clone of the node with all modifications applied to it and an empty mods vec
    /// CAUTION! nodes that point to the original node won't point to the "with_mods" clone!
    /// The clone will also have an empty return_pts vec.
    #[must_use]
    pub fn modded_clone(&self) -> Self {
        // The node is initialized with empty return_pts and mods vecs.
        // (empty return_pts since nothing will actually be pointing to the clone).
        let mut new_node = Self::new(self.value.clone(), self.tree_ptr);
        new_node.parent = self.parent;
        new_node.left = self.left;
        new_node.right = self.right;

        for modifier in &self.mods {
            new_node.apply_mod(modifier)
        }

        new_node
    }

    /// Returns a snapshot of the node at a certain version of the tree
    #[must_use]
    pub fn snapshot(&self, version: usize) -> Self {
        let mut new_node = Self::new(self.value.clone(), self.tree_ptr);
        new_node.parent = self.parent;
        new_node.left = self.left;
        new_node.right = self.right;

        for modifier in &self.mods {
            if modifier.version <= version {
                new_node.apply_mod(modifier)
            } else {
                break;
            }
        }

        new_node
    }

    /// Applies a modification to a node
    #[inline]
    pub fn apply_mod(&mut self, modifier: &Modification<T>) {
        match modifier.modification.clone() {
            ModTarget::Parent(ptr) => self.parent = ptr,
            ModTarget::Left(ptr) => self.left = ptr,
            ModTarget::Right(ptr) => self.right = ptr,
            ModTarget::Value(v) => self.value = v,
        }
    }

    /// Converts node into a heap-allocated link
    #[inline]
    pub fn into_link(self) -> Link<T> {
        // NonNull::new already returns an option (because of the null check),
        // so we don't need to wrap it into a Some()
        NonNull::new(Box::into_raw(Box::new(self)))
    }

    /// Returns a NonNull pointer to the node
    #[inline]
    pub fn as_ptr(&self) -> NodePtr<T> {
        NonNull::from(self)
    }
}
