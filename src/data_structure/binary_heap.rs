use std::cmp::Ordering;

// The BinaryHeap on std does not allow specifying a custom comparator.
// A custom comparator can carry runtime information where Ord implementation cannot.
// It's a min-heap, popping gives the smallest element. Inverting the comparator gives max-heap.
pub struct MyMinHeap<'a, T, Comparator>
    where
        Comparator: Fn(&T, &T) -> Ordering,
{
    data: Vec<T>,
    comparator: &'a Comparator,
}

impl<'a, T, Comparator> MyMinHeap<'a, T, Comparator>
    where
        Comparator: Fn(&T, &T) -> Ordering,
{
    pub fn new(comparator: &'a Comparator) -> Self {
        Self {
            data: Vec::new(),
            comparator,
        }
    }
    
    // the binary heap treats an array as a tree
    // the root is at index 0
    // the left child of a node at index i is at index 2i+1
    // the right child of a node at index i is at index 2i+2
    // the parent of a node at index i is at index (i-1)/2
    // it needs to ensure that the parent is smaller or equal than both children
    // parent <= left_child, parent <= right_child
    
    fn left_child_index(&self, index: usize) -> usize {
        2 * index + 1
    }
    
    fn right_child_index(&self, index: usize) -> usize {
        2 * index + 2
    }
    
    fn parent_index(&self, index: usize) -> usize {
        assert!(index > 0);
        (index - 1) / 2
    }
    
    fn has_node(&self, index: usize) -> bool {
        index < self.data.len()
    }
    
    fn is_root(&self, curr_index: usize) -> bool {
        curr_index == 0
    }
    
    fn check_valid(&self) {
        for i in 1..self.data.len() {
            let parent_index = self.parent_index(i);
            assert!((self.comparator)(&self.data[parent_index], &self.data[i]).is_lt());
        }
    }
    
    // when the element at index is larger than its children, sift it down
    fn sift_down(&mut self, index: usize) {
        let mut curr_parent = index;
        
        loop {
            let left_child = self.left_child_index(curr_parent);
            let right_child = self.right_child_index(curr_parent);
            
            // now we consider 3 nodes: curr_parent, left_child (maybe missing), right_child (maybe missing).
            // if there is no child, the heap property is satisfied.
            // if there is only one child, we need to ensure parent <= child, and swap if necessary.
            // if there are two children, we need to ensure parent <= left_child and parent <= right_child,
            // if it violates, we swap parent with the smaller child.
            // (cannot swap parent with the larger child, as it would still violate the heap property)
            
            // this process is equivalent to finding the minimum of the 3 nodes,
            // and swap it with the parent position if it's not parent.
            // after swapping, the heap property is satisfied for the current parent and its children,
            // but it may violate the heap property for the new child, so continue on child.
            
            let mut min_index = curr_parent;
            
            if self.has_node(left_child) &&
                (self.comparator)(&self.data[left_child], &self.data[min_index]) == Ordering::Less {
                min_index = left_child;
            }
            
            if self.has_node(right_child) &&
                (self.comparator)(&self.data[right_child], &self.data[min_index]) == Ordering::Less {
                min_index = right_child;
            }
            
            if min_index == curr_parent {
                break;
            }
            
            self.data.swap(curr_parent, min_index);
            curr_parent = min_index;
        }
    }
    
    // when the element at index is smaller than its parent, sift it up
    fn sift_up(&mut self, index: usize) {
        let mut curr_index = index;
        
        loop {
            if self.is_root(curr_index) {
                break;
            }
            
            let parent_index = self.parent_index(curr_index);
            
            if (self.comparator)(&self.data[parent_index], &self.data[curr_index]) == Ordering::Greater {
                self.data.swap(parent_index, curr_index);
                curr_index = parent_index;
            } else {
                break;
            }
        }
    }
    
    pub fn insert(&mut self, value: T) {
        self.data.push(value);
        
        // the last element may be smaller than its parent
        // sift it up to keep the heap property
        self.sift_up(self.data.len() - 1);
    }
    
    pub fn take_min(&mut self) -> Option<T> {
        if self.data.is_empty() {
            return None;
        }
        
        // remove the first element and move the last element to its position
        let min_taken = self.data.swap_remove(0);
        
        if !self.data.is_empty() {
            // the first element (if exists) may be larger than its children
            // sift it down to keep the heap property
            self.sift_down(0);
        }
        
        Some(min_taken)
    }
    
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
    
    pub fn peek_min(&self) -> Option<&T> {
        self.data.first()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_my_binary_heap() {
        let mut compare = |a: &i32, b: &i32| a.cmp(b);
        let mut heap = MyMinHeap::new(&mut compare);
        
        heap.insert(3);
        heap.check_valid();
        heap.insert(2);
        heap.check_valid();
        heap.insert(1);
        heap.check_valid();
        heap.insert(4);
        heap.check_valid();
        heap.insert(5);
        heap.check_valid();
        heap.insert(6);
        heap.check_valid();
        
        assert_eq!(heap.take_min(), Some(1));
        heap.check_valid();
        assert_eq!(heap.take_min(), Some(2));
        heap.check_valid();
        assert_eq!(heap.take_min(), Some(3));
        heap.check_valid();
        assert_eq!(heap.take_min(), Some(4));
        heap.check_valid();
        assert_eq!(heap.take_min(), Some(5));
        heap.check_valid();
        assert_eq!(heap.take_min(), Some(6));
        heap.check_valid();
        assert_eq!(heap.take_min(), None);
        heap.check_valid();
    }
}