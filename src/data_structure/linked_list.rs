// The standard library's LinkedList's Cursor indirectly borrows the LinkedList,
// but quick sorting on linked list requires swapping, thus require mutable borrow to LinkedList, which is not allowed.
// Implement a linked list using SlotMap, where cursor does not borrow the list.

use std::collections::HashSet;
use std::fmt::Debug;
use std::marker::PhantomData;

use slotmap::{new_key_type, SlotMap};

// A doubly-linked-list implemented using SlotMap.
// Its cursor does not borrow the list, thus allowing safe quick sorting.
pub struct MyLinkedList<T> {
    nodes: SlotMap<NodeKey, Node<T>>,
    head_and_tail: Option<(NodeKey, NodeKey)>,
}

new_key_type! {
    struct NodeKey;
}

struct Node<T> {
    value: T,
    next: Option<NodeKey>,
    prev: Option<NodeKey>,
}

pub struct Cursor<T> {
    key: NodeKey,
    _phantom: PhantomData<T>,
}

// https://github.com/rust-lang/rust/issues/26925
// #[derive(Clone, Copy)] does not work for that
impl<T> Copy for Cursor<T> {}

impl<T> Clone for Cursor<T> {
    fn clone(&self) -> Cursor<T> {
        *self
    }
}

impl<T> PartialEq<Self> for Cursor<T> {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl<T> Eq for Cursor<T> {}

impl<T> Debug for Cursor<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Cursor({:?})", self.key)
    }
}

impl<T> Cursor<T> {
    fn internal_new(key: NodeKey) -> Cursor<T> {
        Cursor {
            key,
            _phantom: PhantomData,
        }
    }
}

//noinspection DuplicatedCode
impl<T> MyLinkedList<T> {
    pub fn new() -> MyLinkedList<T> {
        MyLinkedList {
            nodes: SlotMap::with_key(),
            head_and_tail: None,
        }
    }
    
    pub fn push_back(&mut self, value: T) -> Cursor<T> {
        let new_node = Node { value, next: None, prev: None };
        let new_key = self.nodes.insert(new_node);
        match self.head_and_tail {
            None => {
                self.head_and_tail = Some((new_key, new_key));
            }
            Some((head, tail)) => {
                self.link(tail, new_key);
                self.head_and_tail = Some((head, new_key));
            }
        }
        Cursor::internal_new(new_key)
    }
    
    pub fn push_front(&mut self, value: T) -> Cursor<T> {
        let new_node = Node { value, next: None, prev: None };
        let new_key = self.nodes.insert(new_node);
        match self.head_and_tail {
            None => {
                self.head_and_tail = Some((new_key, new_key));
            }
            Some((head, tail)) => {
                self.link(new_key, head);
                self.head_and_tail = Some((new_key, tail));
            }
        }
        Cursor::internal_new(new_key)
    }
    
    pub fn insert_after(&mut self, cursor: Cursor<T>, value: T) -> Cursor<T> {
        let new_node = Node { value, next: None, prev: None };
        let new_key = self.nodes.insert(new_node);
        let cursor_next = self.nodes[cursor.key].next;
        
        // (cursor -> cursor_next) becomes (cursor -> new_key -> cursor_next)
        
        self.link(cursor.key, new_key);
        if let Some(cursor_next) = cursor_next {
            self.link(new_key, cursor_next);
        } else {
            // cursor was the tail. change the tail
            let (head, _tail) = self.head_and_tail.unwrap();
            self.head_and_tail = Some((head, new_key));
        }
        Cursor::internal_new(new_key)
    }
    
    pub fn insert_before(&mut self, cursor: Cursor<T>, value: T) -> Cursor<T> {
        let new_node = Node { value, next: None, prev: None };
        let new_key = self.nodes.insert(new_node);
        let cursor_prev = self.nodes[cursor.key].prev;
        
        // (cursor_prev -> cursor) becomes (cursor_prev -> new_key -> cursor)
        
        self.link(new_key, cursor.key);
        if let Some(cursor_prev) = cursor_prev {
            self.link(cursor_prev, new_key);
        } else {
            // cursor was the head. change the head
            let (_head, tail) = self.head_and_tail.unwrap();
            self.head_and_tail = Some((new_key, tail));
        }
        Cursor::internal_new(new_key)
    }
    
    fn link(&mut self, left: NodeKey, right: NodeKey) {
        self.nodes.get_mut(left).unwrap().next = Some(right);
        self.nodes.get_mut(right).unwrap().prev = Some(left);
    }
    
    pub fn begin(&self) -> Option<Cursor<T>> {
        match self.head_and_tail {
            None => None,
            Some((head, _tail)) => Some(Cursor::internal_new(head)),
        }
    }
    
    pub fn end(&self) -> Option<Cursor<T>> {
        match self.head_and_tail {
            None => None,
            Some((_head, tail)) => Some(Cursor::internal_new(tail)),
        }
    }
    
    // it will return None if the cursor is invalid
    pub fn remove_at(&mut self, cursor: Cursor<T>) -> Option<T> {
        let key = cursor.key;
        let node = self.nodes.remove(key)?;
        match (node.prev, node.next) {
            (None, None) => {
                // node was the only node in the list
                self.head_and_tail = None;
            }
            (None, Some(next)) => {
                // node was the head
                let (_curr_head, curr_tail) = self.head_and_tail.unwrap();
                self.head_and_tail = Some((next, curr_tail));
                self.nodes[next].prev = None;
            }
            (Some(prev), None) => {
                // node was the tail
                let (curr_head, _curr_tail) = self.head_and_tail.unwrap();
                self.head_and_tail = Some((curr_head, prev));
                self.nodes[prev].next = None;
            }
            (Some(prev), Some(next)) => {
                // node was in the middle
                self.link(prev, next);
            }
        }
        Some(node.value)
    }
    
    // it will panic if an invalid cursor is given
    pub fn borrow(&self, cursor: Cursor<T>) -> &T {
        &self.nodes[cursor.key].value
    }
    
    // only supports borrowing one element at a time
    // (disjoint borrowing is more complicated)
    // it will panic if an invalid cursor is given
    pub fn borrow_mut(&mut self, cursor: Cursor<T>) -> &mut T {
        &mut self.nodes[cursor.key].value
    }
    
    // it returns true if swap succeeded
    // it can succeed only if a and b are valid cursors and are not equal
    pub fn swap(&mut self, a: Cursor<T>, b: Cursor<T>) -> bool {
        let keys: [NodeKey; 2] = [a.key, b.key];
        let refs: Option<[&mut Node<T>; 2]> = self.nodes.get_disjoint_mut(keys);
        if let Some([a, b]) = refs {
            std::mem::swap(&mut a.value, &mut b.value);
            true
        } else {
            false
        }
    }
    
    pub fn next_cursor(&self, cursor: Cursor<T>) -> Option<Cursor<T>> {
        self.nodes[cursor.key].next.map(Cursor::internal_new)
    }
    
    pub fn prev_cursor(&self, cursor: Cursor<T>) -> Option<Cursor<T>> {
        self.nodes[cursor.key].prev.map(Cursor::internal_new)
    }
    
    pub fn size(&self) -> usize {
        self.nodes.len()
    }
    
    fn check_valid(&self) {
        if let None = self.head_and_tail {
            assert!(self.nodes.is_empty());
            return;
        }
        
        assert_eq!(self.nodes[self.head_and_tail.unwrap().0].prev, None);
        assert_eq!(self.nodes[self.head_and_tail.unwrap().1].next, None);
        
        let mut visited = HashSet::new();
        let mut cursor = self.begin();
        while let Some(c) = cursor {
            let key = c.key;
            assert!(!visited.contains(&key));
            visited.insert(key);
            let next_cursor = self.next_cursor(c);
            if let Some(next_cursor) = next_cursor {
                assert_eq!(self.prev_cursor(next_cursor), Some(c));
            }
            cursor = next_cursor;
        }
        
        assert_eq!(visited.len(), self.nodes.len());
    }
    
    pub fn iter(&self) -> MyLinkedListIter<T> {
        MyLinkedListIter::new(self)
    }
}

pub struct MyLinkedListIter<'a, T> {
    list: &'a MyLinkedList<T>,
    cursor: Option<Cursor<T>>,
}

impl<T> MyLinkedListIter<'_, T> {
    pub fn new(list: &MyLinkedList<T>) -> MyLinkedListIter<T> {
        MyLinkedListIter {
            list,
            cursor: list.begin(),
        }
    }
}

impl<'a, T> Iterator for MyLinkedListIter<'a, T> {
    type Item = &'a T;
    
    fn next(&mut self) -> Option<&'a T> {
        let cursor = self.cursor?;
        let value = self.list.borrow(cursor);
        self.cursor = self.list.next_cursor(cursor);
        Some(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_my_linked_list() {
        let mut list = MyLinkedList::new();
        list.check_valid();
        let a = list.push_back(1);
        list.check_valid();
        let b = list.push_back(2);
        list.check_valid();
        let c = list.push_front(3);
        list.check_valid();
        let d = list.push_back(4);
        list.check_valid();
        let e = list.push_front(5);
        list.check_valid();
        assert_eq!(list.remove_at(a), Some(1));
        list.check_valid();
        assert_eq!(list.remove_at(e), Some(5));
        list.check_valid();
        assert_eq!(list.remove_at(c), Some(3));
        list.check_valid();
        let f = list.push_back(6);
        list.check_valid();
        let g = list.insert_before(d, 7);
        list.check_valid();
        let h = list.insert_after(b, 8);
        list.check_valid();
    }
}