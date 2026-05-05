// #![allow(dead_code)]

pub mod persistent_btree;

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

        for i in 0..=tree.version() {
            println!("====== TREE AT VERSION {i} ======");
            println!("{:?}", tree.successor(&10, i));
            // println!("{}", tree.list(i));
        }

    }
}
