use std::{marker::PhantomData, ptr::NonNull};

pub struct List<T> {
    head: Link<T>,
    tail: Link<T>,
    _t: PhantomData<T>,
}

type Link<T> = Option<NonNull<Node<T>>>;
struct Node<T> {
    elem: T,
    next: Link<T>,
}

impl<T> Node<T> {
    fn new(elem: T, next: Link<T>) -> Self {
        Self { elem, next }
    }
}

impl<T> List<T> {
    pub fn new() -> Self {
        Self {
            head: None,
            tail: None,
            _t: PhantomData,
        }
    }

    pub fn push(&mut self, elem: T) {
        let boxed_tail = Box::new(Node::new(elem, None));
        let new_tail = NonNull::new(Box::into_raw(boxed_tail));

        // Tries appending the element to the tail, sets head instead if empty list
        if let Some(mut node) = self.tail.take() {
            unsafe { node.as_mut().next = new_tail }
        } else {
            self.head = new_tail
        }

        self.tail = new_tail
    }

    pub fn pop(&mut self) -> Option<T> {
        if let Some(node) = self.head.take() {
            // "Box::from_raw to clean up the allocation" (?)
            let head = unsafe { Box::from_raw(node.as_ptr()) };

            // Swaps head to the next element
            self.head = head.next;
            // If head is none, the popped element was the last in the list
            if self.head.is_none() {
                self.tail = None
            }

            Some(head.elem)
        } else {
            None
        }
    }
}

impl<T> Drop for List<T> {
    fn drop(&mut self) {
        // The popped elements will be dropped automatically
        while let Some(_) = self.pop() {}
    }
}


#[cfg(test)]
mod test {
    use super::List;
    #[test]
    fn basics() {
        let mut list = List::new();

        // Check empty list behaves right
        assert_eq!(list.pop(), None);

        // Populate list
        list.push(1);
        list.push(2);
        list.push(3);

        // Check normal removal
        assert_eq!(list.pop(), Some(1));
        assert_eq!(list.pop(), Some(2));

        // Push some more just to make sure nothing's corrupted
        list.push(4);
        list.push(5);

        // Check normal removal
        assert_eq!(list.pop(), Some(3));
        assert_eq!(list.pop(), Some(4));

        // Check exhaustion
        assert_eq!(list.pop(), Some(5));
        assert_eq!(list.pop(), None);

        // Check the exhaustion case fixed the pointer right
        list.push(6);
        list.push(7);

        // Check normal removal
        assert_eq!(list.pop(), Some(6));
        assert_eq!(list.pop(), Some(7));
        assert_eq!(list.pop(), None);
    }
}
