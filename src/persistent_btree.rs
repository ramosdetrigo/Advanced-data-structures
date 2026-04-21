#![allow(dead_code)]
use std::ptr::NonNull;

type Link<T> = Option<NonNull<Node<T>>>;

pub struct PBTree<T>
where
    T: PartialOrd,
{
    root: Link<T>,
    version: usize,
    // TODO: how tf do I implement modifications on the root??
}

struct Node<T>
where
    T: PartialOrd,
{
    value: T,
    parent: Link<T>,
    left: Link<T>,
    right: Link<T>,
    return_pts: [Link<T>; 3], // P = 3
    mods: [Modification<T>; 6], // 2P = 6
}

struct Modification<T>
where
    T: PartialOrd,
{
    version: usize,
    modification: ModType<T>,
}

enum ModType<T>
where
    T: PartialOrd,
{
    Value(T),
    Parent(Link<T>),
    Left(Link<T>),
    Right(Link<T>),
}
