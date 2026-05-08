use std::fmt::Debug;

use super::node::*;

#[derive(Clone, Debug)]
pub struct Modification<K, V>
where
    K: PartialOrd + Clone + Debug,
    V: Clone + Debug,
{
    pub version: usize,
    pub modification: ModTarget<K, V>,
}

impl<K, V> Modification<K, V>
where
    K: PartialOrd + Clone + Debug,
    V: Clone + Debug,
{
    #[inline]
    #[must_use]
    pub fn parent(link: Link<K, V>, version: usize) -> Self {
        Self {
            version,
            modification: ModTarget::Parent(link),
        }
    }

    #[inline]
    #[must_use]
    pub fn left(link: Link<K, V>, version: usize) -> Self {
        Self {
            version,
            modification: ModTarget::Left(link),
        }
    }

    #[inline]
    #[must_use]
    pub fn right(link: Link<K, V>, version: usize) -> Self {
        Self {
            version,
            modification: ModTarget::Right(link),
        }
    }

    #[inline]
    #[must_use]
    pub fn value(value: V, version: usize) -> Self {
        Self {
            version,
            modification: ModTarget::Value(value),
        }
    }

    #[inline]
    #[must_use]
    pub fn key(key: K, version: usize) -> Self {
        Self {
            version,
            modification: ModTarget::Key(key),
        }
    }
}

/// Represents a modification to a specific field of the node
#[derive(Clone, Debug)]
pub enum ModTarget<K, V>
where
    K: PartialOrd + Clone + Debug,
    V: Clone + Debug,
{
    Parent(Link<K, V>),
    Left(Link<K, V>),
    Right(Link<K, V>),
    Value(V),
    Key(K),
}

/// A return pointer to another node. \
/// This reads as "this node is Left/Right/Parent of another node": \
/// `ReturnPtr::LeftOf(Y)` inside `X.return_pts` would mean `Y.left = X`
#[derive(Clone, PartialEq, Debug)]
pub enum ReturnPtr<K, V>
where
    K: PartialOrd + Clone + Debug,
    V: Clone + Debug,
{
    ParentOf(NodePtr<K, V>),
    LeftOf(NodePtr<K, V>),
    RightOf(NodePtr<K, V>),
}

impl<K, V> ReturnPtr<K, V>
where
    K: PartialOrd + Clone + Debug,
    V: Clone + Debug,
{
    #[inline]
    #[must_use]
    /// "Unwraps" the ReturnPtr
    pub fn ptr(&self) -> NodePtr<K, V> {
        match self {
            ReturnPtr::LeftOf(ptr) => *ptr,
            ReturnPtr::RightOf(ptr) => *ptr,
            ReturnPtr::ParentOf(ptr) => *ptr,
        }
    }

    #[inline]
    /// Sets the ReturnPtr pointer into a new one
    pub fn set_ptr(&mut self, new_ptr: NodePtr<K, V>) {
        match self {
            ReturnPtr::LeftOf(ptr) => *ptr = new_ptr,
            ReturnPtr::RightOf(ptr) => *ptr = new_ptr,
            ReturnPtr::ParentOf(ptr) => *ptr = new_ptr,
        };
    }
}
