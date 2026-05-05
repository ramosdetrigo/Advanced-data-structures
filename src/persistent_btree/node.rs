use super::*;

type NodePtr<T> = NonNull<Node<T>>;
pub type Link<T> = Option<NodePtr<T>>;

const P: usize = 3; // Número de ponteiros em um nó
#[derive(Clone)]
pub struct Node<T>
where
    T: PartialOrd + Clone,
{
    value: T,
    parent: Link<T>,
    left: Link<T>,
    right: Link<T>,
    // NOTE: Since we're using a vec, the P and 2P restrictions need to be enforced via code!
    return_pts: Vec<ReturnPtr<T>>, // P = 3
    mods: Vec<Modification<T>>,    // 2P = 6
}

#[allow(dead_code)]
pub struct NodeSnapshot<T>
where
    T: PartialOrd + Clone,
{
    pub value: T,
    pub parent: Link<T>,
    pub left: Link<T>,
    pub right: Link<T>,
}

#[derive(Clone)]
pub struct Modification<T>
where
    T: PartialOrd + Clone,
{
    version: usize,
    modification: ModTarget<T>,
}

#[allow(dead_code)]
impl<T> Modification<T>
where
    T: PartialOrd + Clone,
{
    pub fn parent(link: Link<T>, version: usize) -> Self {
        Self {
            version,
            modification: ModTarget::Parent(link),
        }
    }

    pub fn left(link: Link<T>, version: usize) -> Self {
        Self {
            version,
            modification: ModTarget::Left(link),
        }
    }

    pub fn right(link: Link<T>, version: usize) -> Self {
        Self {
            version,
            modification: ModTarget::Right(link),
        }
    }

    pub fn value(value: T, version: usize) -> Self {
        Self {
            version,
            modification: ModTarget::Value(value),
        }
    }
}

/// Represents a modification to a specific field of the node
#[derive(Clone)]
pub enum ModTarget<T>
where
    T: PartialOrd + Clone,
{
    Parent(Link<T>),
    Left(Link<T>),
    Right(Link<T>),
    Value(T),
}

/// A return pointer to another node. \
/// This reads as "this node is Left/Right/Parent of another node": \
/// `ReturnPtr::LeftOf(Y)` inside `X.return_pts` would mean `Y.left = X`
#[derive(Clone, PartialEq)]
pub enum ReturnPtr<T>
where
    T: PartialOrd + Clone,
{
    ParentOf(NodePtr<T>),
    LeftOf(NodePtr<T>),
    RightOf(NodePtr<T>),
}

impl<T> ReturnPtr<T>
where
    T: PartialOrd + Clone,
{
    /// Adds a modifier to the return_ptr's node to point to the new node
    pub fn redirect_node(&mut self, version: usize, new_target: Link<T>) {
        let (return_ptr, modification) = match self {
            ReturnPtr::LeftOf(ptr) => (ptr, ModTarget::Left(new_target)),
            ReturnPtr::RightOf(ptr) => (ptr, ModTarget::Right(new_target)),
            ReturnPtr::ParentOf(ptr) => (ptr, ModTarget::Parent(new_target)),
        };

        unsafe {
            return_ptr.as_mut().add_mod(Modification {
                version,
                modification,
            });
        };
    }
}

impl<T> Node<T>
where
    T: PartialOrd + Clone,
{
    // Node constructor
    pub fn new(value: T) -> Self {
        Self {
            value,
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
    pub fn add_mod(&mut self, modifier: Modification<T>) -> Link<T> {
        // Constraint: the mods field max size is P * 2
        if self.mods.len() == P * 2 {
            Some(self.add_mod_full(modifier))
        } else {
            self.add_mod_empty(modifier, None);
            None
        }
    }

    fn add_mod_empty(&mut self, modifier: Modification<T>, old_node: Link<T>) {
        // Ponteiro pro nó antigo. Aponta pro nó atual se old_node = None
        // Útil pro add_mod_full não causar muita confusão.
        let old_node_ptr = old_node.unwrap_or(self.as_ptr());

        // Case 1: value update
        if let ModTarget::Value(_) = modifier.modification {
            self.mods.push(modifier);
        // Case 2: pointer update
        } else {
            let current = self.with_mods();

            // old_ptr: Ponteiro pro nó antigo do campo modificado
            // old_ret_ptr: Ponteiro de retorno pro nó antigo
            // new_ret_ptr: Ponteiro de retorno pro nó novo
            // new_ptr: Novo ponteiro do campo modificado
            let (old_ptr, old_ret_ptr, new_ret_ptr, new_ptr) = match modifier.modification {
                ModTarget::Right(ptr) => (
                    current.right,
                    ReturnPtr::RightOf(old_node_ptr),
                    ReturnPtr::RightOf(self.as_ptr()),
                    ptr,
                ),
                ModTarget::Left(ptr) => (
                    current.left,
                    ReturnPtr::LeftOf(old_node_ptr),
                    ReturnPtr::LeftOf(self.as_ptr()),
                    ptr,
                ),
                ModTarget::Parent(ptr) => (
                    current.parent,
                    ReturnPtr::ParentOf(old_node_ptr),
                    ReturnPtr::ParentOf(self.as_ptr()),
                    ptr,
                ),
                _ => unreachable!(),
            };

            // Adiciona o mod
            self.mods.push(modifier);

            // Remove o nó antigo dos ponteiros de retorno do nó anterior do campo modificado
            if let Some(mut node) = old_ptr {
                let node = unsafe { node.as_mut() };
                node.return_pts.retain(|ptr| *ptr != old_ret_ptr);
            }

            // Adiciona o nó atual nos ponteiros de retorno do novo nó do campo modificado
            if let Some(mut node) = new_ptr {
                let node = unsafe { node.as_mut() };
                node.add_return_ptr(new_ret_ptr);
            }
        }
    }

    /// Handles the case for add_mod() where the mods field is full.
    #[must_use]
    fn add_mod_full(&mut self, modifier: Modification<T>) -> NodePtr<T> {
        // Create node copy with empty mods field
        let mut new_node = self.with_mods();

        // Move return pointers from the old node to the new one, since things
        // will now be pointing to the new node instead of the old one.
        new_node.return_pts = std::mem::take(&mut self.return_pts);

        // Adds the modifier to the new node, erasing the return pointer from the old node if necessary.
        let version = modifier.version; // copy version 'cause modifier is getting moved
        new_node.add_mod_empty(modifier, Some(self.as_ptr()));

        // Save a copy of the return pointers for updating the nodes
        let return_pts = new_node.return_pts.clone();

        // Then convert new_node into a heap-allocated link
        let new_node = new_node.into_link();

        // Finally, update the nodes that pointed to the old node to point to the new one
        for mut return_ptr in return_pts {
            return_ptr.redirect_node(version, new_node);
        }

        // Returns a pointer to the new node
        new_node.unwrap()
    }

    /// Transplants a node: Switches the node's parent's child from self to target.
    pub fn transplant(&mut self, target: Link<T>, version: usize) {
        // Get latest parent link
        let mut parent_link = self.with_mods().parent;

        // Updates parent to point to target instead of self
        if let Some(mut parent_ptr) = parent_link {
            // Gets parent and updated parent
            let parent = unsafe { parent_ptr.as_mut() };
            let current_parent = parent.with_mods();

            // Check if we're the parent's left or right child
            let modifier = if current_parent.left.is_some_and(|ptr| ptr == self.as_ptr()) {
                Modification::left(target, version)
            // Else we're the parent's right child
            } else {
                Modification::right(target, version)
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
        if let Some(mut ptr) = target {
            let target_node = unsafe { ptr.as_mut() };
            target_node.add_mod(Modification::parent(parent_link, version));
        }
    }

    pub fn remove(&mut self, version: usize) {
        let current_self = self.with_mods();

        if current_self.left.is_none() {
            self.transplant(current_self.right, version);
        } else if current_self.right.is_none() {
            self.transplant(current_self.left, version);
        } else {
            // we can unwrap because we're guaranteed to have at least a right child.
            let mut successor_ptr = self.with_mods().latest_successor().unwrap();
            let successor_link = Some(successor_ptr);
            let successor = unsafe { successor_ptr.as_mut() };

            // Remove successor from its place
            successor.transplant(successor.right, version);

            // Override current_self after the transplant (right could be None)
            let current_self = self.with_mods();

            // Updates left child
            successor.add_mod(Modification::left(current_self.left, version));
            unsafe {
                current_self
                    .left
                    .unwrap()
                    .as_mut()
                    .add_mod(Modification::parent(successor_link, version))
            };

            // Updates successor's right and self.right's parent (if self.right != null)
            successor.add_mod(Modification::right(current_self.right, version));
            if current_self.right.is_some() {
                unsafe {
                    current_self
                        .right
                        .unwrap()
                        .as_mut()
                        .add_mod(Modification::parent(successor_link, version))
                };
            }

            self.transplant(successor_link, version);
        }
    }

    pub fn latest_successor(&self) -> Link<T> {
        let mut successor: Link<T> = self.right;
        let mut curr_node = self.right;
        while let Some(node) = curr_node {
            let latest = unsafe { node.as_ref().with_mods() };
            // If the value of the node is greater than the element,
            // update successor & search to the left
            if latest.value > self.value {
                successor = Some(node);
                curr_node = latest.left;
            // Else: search to the right
            } else {
                curr_node = latest.right;
            }
        }
        successor
    }

    pub fn add_return_ptr(&mut self, ret_ptr: ReturnPtr<T>) {
        assert!(
            self.return_pts.len() < P,
            "Return pointer vec length exceeded {}!",
            P
        );
        self.return_pts.push(ret_ptr);
    }

    /// Returns a clone of the node with all modifications applied to it and an empty mods vec
    /// CAUTION! nodes that point to the original node won't point to the "with_mods" clone!
    pub fn with_mods(&self) -> Self {
        // The node is initialized with empty return_pts and mods vecs.
        // (empty return_pts since nothing will actually be pointing to the clone).
        let mut new_node = Self::new(self.value.clone());
        new_node.parent = self.parent;
        new_node.left = self.left;
        new_node.right = self.right;

        for modifier in &self.mods {
            new_node.apply_mod(modifier)
        }

        new_node
    }

    /// Returns a snapshot (a "read-only copy") of the node
    /// at a certain version of the tree
    pub fn at_version(&self, version: usize) -> NodeSnapshot<T> {
        let mut new_node = Self::new(self.value.clone());
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

        NodeSnapshot {
            value: new_node.value,
            parent: new_node.parent,
            left: new_node.left,
            right: new_node.right,
        }
    }

    /// Applies a modification to a node
    pub fn apply_mod(&mut self, modifier: &Modification<T>) {
        match modifier.modification.clone() {
            ModTarget::Parent(ptr) => self.parent = ptr,
            ModTarget::Left(ptr) => self.left = ptr,
            ModTarget::Right(ptr) => self.right = ptr,
            ModTarget::Value(v) => self.value = v,
        }
    }

    /// Converts node into a heap-allocated link
    pub fn into_link(self) -> Link<T> {
        // NonNull::new already returns an option (because of the null check),
        // so we don't need to wrap it into a Some()
        NonNull::new(Box::into_raw(Box::new(self)))
    }

    /// Returns a NonNull pointer to the node
    pub fn as_ptr(&self) -> NodePtr<T> {
        NonNull::from(self)
    }
}
