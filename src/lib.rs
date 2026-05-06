pub mod persistent_btree;

#[cfg(test)]
mod pbtree_test {
    use super::persistent_btree::PBTree;
    #[test]
    fn basic() {
        let mut tree: PBTree<i32> = PBTree::new();
        tree.push(1);
        tree.push(3);//1
        tree.push(4);
        tree.push(5);
        tree.push(6);
        tree.remove(&3);//2
        tree.remove(&4);//3
        tree.remove(&5);//4
        tree.remove(&6);//5
        tree.push(2);//6
        tree.push(3);//1
        tree.remove(&3);//2
        tree.push(3);//3
        tree.remove(&3);//4
        tree.push(3);//5
        tree.remove(&3);//6
        tree.push(3);//7



        for i in 1..=tree.version() {
            println!("====== TREE AT VERSION {i} ======");
            println!("{}", tree.list_tree(i));
        }
    }
}
