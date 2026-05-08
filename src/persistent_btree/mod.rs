mod node;
mod tools;

use node::*;
use std::{fmt::Debug, marker::PhantomData, ptr::NonNull};

#[derive(Debug)]
pub struct BinTreeMap<K, V>
where
    K: PartialOrd + Clone + Debug,
    V: Clone + Debug,
{
    root: Option<NodeHandle<K, V>>,
    root_history: Vec<(NodeSnapshot<K, V>, usize)>,
    version: usize,
    _t: PhantomData<Box<(K, V)>>,
}

impl<K, V> BinTreeMap<K, V>
where
    K: PartialOrd + Clone + Debug,
    V: Clone + Debug,
{
    #[must_use]
    pub fn new() -> Self {
        Self {
            root: None,
            root_history: Vec::new(),
            version: 0,
            _t: PhantomData,
        }
    }

}

// TODO: impl Drop :P
// i <3 memory leaks
