use std::mem;

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
    /// Adds a modifier to the return_ptr's node to point to a new node
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

    pub fn add_mod(&mut self, modifier: Modification<T>) {
        // Constraint: the mods field max size is P * 2
        if self.mods.len() == P * 2 {
            self.add_mod_full(modifier);
            return;
        }

        // Case 1: value update
        if let ModTarget::Value(_) = modifier.modification {
            self.mods.push(modifier);
        // Case 2: pointer update
        } else {
            let current = self.with_mods();

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
                node.return_pts.retain(|ptr| *ptr != ret_ptr);
            }

            // Adiciona o nó atual nos ponteiros de retorno do novo nó
            if let Some(mut node) = new_ptr {
                let node = unsafe { node.as_mut() };
                node.add_return_ptr(ret_ptr);
            }
        }
    }

    pub fn add_return_ptr(&mut self, ret_ptr: ReturnPtr<T>) {
        assert!(
            self.return_pts.len() < P,
            "Return pointer vec lenght exceeded {}!",
            P
        );
        self.return_pts.push(ret_ptr);
    }

    fn add_mod_full(&mut self, modifier: Modification<T>) {
        // Create node copy with empty mods field
        let mut new_node = self.with_mods();

        // Move return pointers from the old node to the new one, since things
        // will now be pointing to the new node instead of the old one.
        new_node.return_pts = mem::take(&mut self.return_pts);

        // Save a copy of the return pointers for updating the nodes
        let return_pts = new_node.return_pts.clone();

        // Then convert new_node into a heap-allocated link
        let new_node = new_node.into_link();

        // And update nodes that pointed to the old node to point to the new one
        for mut return_ptr in return_pts {
            return_ptr.redirect_node(modifier.version, new_node);
        }

        unsafe { new_node.unwrap().as_mut() }.add_mod(modifier);
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

    pub fn at_version(&self, version: usize) -> NodeSnapshot<T> {
        let mut new_node = Self::new(self.value.clone());
        new_node.parent = self.parent;
        new_node.left = self.left;
        new_node.right = self.right;

        for modifier in &self.mods {
            if modifier.version <= version {
                new_node.apply_mod(modifier)
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
