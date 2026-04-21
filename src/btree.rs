use std::{fmt::Display, marker::PhantomData, ptr::NonNull};

type Link<T> = Option<NonNull<Node<T>>>;

pub struct BTree<T>
where
    T: PartialOrd,
{
    root: Link<T>,
    _t: PhantomData<T>,
}

struct Node<T>
where
    T: PartialOrd,
{
    pub value: T,
    #[allow(dead_code)]
    pub parent: Link<T>,
    pub left: Link<T>,
    pub right: Link<T>,
}

impl<T> Node<T>
where
    T: PartialOrd,
{
    pub fn new(value: T, parent: Link<T>, left: Link<T>, right: Link<T>) -> Self {
        Self {
            value,
            parent,
            left,
            right,
        }
    }
}

impl<T> BTree<T>
where
    T: PartialOrd + Display,
{
    pub fn list(&self) -> String {
        let mut stack: Vec<(NonNull<Node<T>>, usize)> = Vec::new();
        let mut depth = 0;
        let mut curr_node = self.root;
        let mut result = String::new();

        while curr_node.is_some() || !stack.is_empty() {
            // Go as left as possible
            while let Some(node_ptr) = curr_node {
                stack.push((node_ptr, depth));
                let node = unsafe { node_ptr.as_ref() };
                curr_node = node.left;
                depth += 1;
            }

            // Read current node
            let (node_ptr, node_depth) = stack.pop().unwrap();
            let node_ref = unsafe { node_ptr.as_ref() };
            result.push_str(&format!("(v: {}, d: {})\n", node_ref.value, node_depth));

            // Read right node
            curr_node = node_ref.right;
            depth = node_depth + 1
        }

        result
    }
}

impl<T> BTree<T>
where
    T: PartialOrd,
{
    pub fn new() -> Self {
        Self {
            root: None,
            _t: PhantomData,
        }
    }

    /// Finds the smallest strictly greater than X element in the tree
    pub fn successor(&self, elem: &T) -> Option<&T> {
        let mut successor: Option<&T> = None;
        let mut curr_node = self.root;
        while let Some(node) = curr_node {
            let node = unsafe { node.as_ref() };
            // If the value of the node is greater than the element,
            // update successor & search to the left
            if node.value > *elem {
                successor = Some(&node.value);
                curr_node = node.left;
            // Else: search to the right
            } else {
                curr_node = node.right;
            }
        }
        successor
    }

    /// Finds the node that would be the parent of an element in the tree
    fn find_parent_node(&self, elem: &T) -> Link<T> {
        let mut prev_node: Link<T> = None;
        let mut curr_node = self.root;

        // Searches the correct place to add the node
        while let Some(node) = curr_node {
            prev_node = curr_node;
            let node = unsafe { node.as_ref() };
            if *elem < node.value {
                curr_node = node.left
            } else {
                curr_node = node.right
            }
        }

        prev_node
    }

    pub fn push(&mut self, elem: T) {
        let parent_link = self.find_parent_node(&elem);

        if let Some(mut parent) = parent_link {
            // Accesses the parent's data and checks if the node is a left or right child
            let parent = unsafe { parent.as_mut() };
            let is_left_child: bool = elem < parent.value;

            // Creates the new node after the check (prevents moving elem)
            let new_node = Node::new(elem, parent_link, None, None);
            // NonNull::new already returns an option (because of the null check).
            // So we don't need to wrap it into a Some()
            let link = NonNull::new(Box::into_raw(Box::new(new_node)));

            if is_left_child {
                parent.left = link;
            } else {
                parent.right = link;
            }
        } else {
            // Edge case: tree is empty
            let new_node = Node::new(elem, None, None, None);
            self.root = NonNull::new(Box::into_raw(Box::new(new_node)))
        }
    }
}
