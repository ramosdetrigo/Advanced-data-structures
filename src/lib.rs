// #![allow(dead_code)]

pub mod persistent_btree;
pub mod btree;
// pub mod queue;

#[cfg(test)]
mod pbtree_test {
    use super::persistent_btree::PBTree;
    #[test]
    fn basic() {
        let mut tree: PBTree<i32> = PBTree::new();
        tree.push(1);
        tree.push(5);
        tree.push(-5);
        tree.push(10);
        tree.push(13);
        tree.push(4);
        tree.push(-3);
        tree.push(-7);
        tree.push(-5);

        println!("{}", tree.list(tree.version()))
    }
}

#[cfg(test)]
mod btree_test {
    use super::btree::BTree;
    #[test]
    fn basic() {
        let mut tree: BTree<i32> = BTree::new();
        tree.push(1);
        tree.push(5);
        tree.push(-5);
        tree.push(10);
        tree.push(13);
        tree.push(4);
        tree.push(-3);
        tree.push(-7);
        tree.push(-5);

        println!("{}", tree.list())
    }
}
