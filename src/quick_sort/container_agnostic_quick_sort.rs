// The container-agnostic quick sort that works both on arrays and linked lists.

use std::cmp::Ordering;

use crate::data_structure::linked_list::{Cursor, MyLinkedList};
use crate::quick_sort::pivot_select;

// The Index is usize for array and node reference for linked list.
// Note: as the algorithm uses Rust's range convention where the right end is exclusive,
// the Index must be able to represent the virtual slot after the last element.
pub trait QuickSortableContainer<Element> {
    type Index: Clone + Eq;
    
    fn swap(&mut self, a: Self::Index, b: Self::Index);
    
    fn get(&self, index: Self::Index) -> &Element;
    
    fn next_index(&self, index: Self::Index) -> Self::Index;
    
    fn prev_index(&self, index: Self::Index) -> Self::Index;
    
    fn select_pivot_index<
        Comparator: Fn(&Element, &Element) -> Ordering
    >(
        &self, range_begin: Self::Index, range_end_exclusive: Self::Index,
        comparator: &Comparator,
    ) -> Self::Index;
}

pub struct PartitionResult<Index> {
    // the left part is container[range_begin..left]
    // the right part is container[right..range_end_exclusive]
    left: Index,
    right: Index,
    left_part_size: usize,
    right_part_size: usize,
}

/// It's based on the fat partition (see [partition::fat_partition_no_clone_required])
pub fn container_agnostic_fat_partition<
    Element, Index: Eq + Clone, Comparator, Container: QuickSortableContainer<Element, Index=Index> + ?Sized
>(
    container: &mut Container,
    comparator: &Comparator,
    range_begin: Index,
    range_end_exclusive: Index,
    initial_pivot_index: Index,
    range_size: usize,
) -> PartitionResult<Index>
    where
        Comparator: Fn(&Element, &Element) -> Ordering
{
    if range_begin == range_end_exclusive {
        return PartitionResult {
            left: range_begin.clone(),
            right: range_end_exclusive.clone(),
            left_part_size: 0,
            right_part_size: 0,
        };
    }
    
    let mut curr_pivot_index = initial_pivot_index;
    
    let mut left_index = range_begin.clone();
    let mut right_index = container.prev_index(range_end_exclusive.clone());
    let mut eq_index = range_begin.clone();
    
    // we need to track the size of the left and right regions,
    // because it's slow to compare node reference order in linked list,
    // so we compute the corresponding integer index using the tracked size.
    // even in linked list, we can still assign integer indices, by tracking the node traversal.
    let mut left_part_size = 0;
    let mut right_part_size = 0;
    let mut left_part_and_eq_part_size = 0;
    // locally in the range:
    // left_index == left_part_size
    // right_index == range_size - 1 - right_part_size
    // eq_index == left_part_and_eq_part_size
    // so, eq_index <= right_index translates to left_part_and_eq_part_size <= range_size - 1 - right_part_size
    // to avoid unsigned integer underflow,
    // convert to left_part_and_eq_part_size + 1 + right_part_size <= range_size
    
    while left_part_and_eq_part_size + 1 + right_part_size <= range_size {
        if curr_pivot_index == eq_index {
            // no need to compare arr[eq_index] with arr[pivot_index] now, treat it as equal
            eq_index = container.next_index(eq_index);
            left_part_and_eq_part_size += 1;
            continue;
        }
        // now curr_pivot_index != eq_index
        
        match comparator(container.get(eq_index.clone()), container.get(curr_pivot_index.clone())) {
            Ordering::Less => {
                if left_index == eq_index {
                    left_index = container.next_index(left_index);
                    eq_index = container.next_index(eq_index);
                    left_part_size += 1;
                    left_part_and_eq_part_size += 1;
                } else {
                    container.swap(eq_index.clone(), left_index.clone());
                    
                    if left_index == curr_pivot_index {
                        // update the pivot index as it has been moved
                        curr_pivot_index = eq_index.clone();
                    }
                    
                    left_index = container.next_index(left_index);
                    eq_index = container.next_index(eq_index);
                    left_part_size += 1;
                    left_part_and_eq_part_size += 1;
                }
            }
            Ordering::Equal => {
                eq_index = container.next_index(eq_index);
                left_part_and_eq_part_size += 1;
            }
            Ordering::Greater => {
                container.swap(eq_index.clone(), right_index.clone());
                
                if right_index == curr_pivot_index {
                    // update the pivot index as it has been moved
                    curr_pivot_index = eq_index.clone();
                }
                
                right_index = container.prev_index(right_index);
                right_part_size += 1;
            }
        }
    }
    
    assert!(eq_index == container.next_index(right_index.clone()));
    
    let r_left = left_index;
    let r_right = container.next_index(right_index);
    assert!(r_left != r_right);
    PartitionResult {
        left: r_left,
        right: r_right,
        left_part_size,
        right_part_size,
    }
}

pub fn container_agnostic_quick_sort<
    Element, Index: Eq + Clone, Comparator, Container: QuickSortableContainer<Element, Index=Index> + ?Sized
>(
    container: &mut Container,
    comparator: &Comparator,
    range_begin: Index,
    range_end_exclusive: Index,
    range_size: usize,
) where
    Comparator: Fn(&Element, &Element) -> Ordering
{
    if range_size <= 1 {
        return;
    }
    
    if range_size == 2 {
        let i0 = range_begin.clone();
        let i1 = container.next_index(range_begin.clone());
        if comparator(container.get(i0.clone()), container.get(i1.clone())) == Ordering::Greater {
            container.swap(i0, i1);
        }
        return;
    }
    
    let initial_pivot_index =
        container.select_pivot_index(range_begin.clone(), range_end_exclusive.clone(), comparator);
    
    let PartitionResult { left, right, left_part_size, right_part_size } =
        container_agnostic_fat_partition(
            container, comparator, range_begin.clone(), range_end_exclusive.clone(), initial_pivot_index,
            range_size,
        );
    
    container_agnostic_quick_sort(container, comparator, range_begin, left, left_part_size);
    
    container_agnostic_quick_sort(container, comparator, right, range_end_exclusive, right_part_size);
}

// slice is quick-sortable
impl<Element> QuickSortableContainer<Element> for [Element] {
    type Index = usize;
    
    fn swap(&mut self, a: usize, b: usize) {
        self.swap(a, b);
    }
    
    fn get(&self, index: usize) -> &Element {
        &self[index]
    }
    
    fn next_index(&self, index: usize) -> usize {
        index + 1
    }
    
    fn prev_index(&self, index: usize) -> usize {
        index - 1
    }
    
    fn select_pivot_index<
        Comparator: Fn(&Element, &Element) -> Ordering
    >(
        &self, range_begin: usize, range_end_exclusive: usize,
        comparator: &Comparator,
    ) -> usize {
        pivot_select::median_of_three_pivot(&self[range_begin..range_end_exclusive], comparator)
            + range_begin
    }
}

// linked list's cursor does not allow representing the slot after the last element,
// so we need to use an enum to represent the index
pub enum LinkedListIndex<Element> {
    Cursor(Cursor<Element>),
    AfterLast,
}

impl<Element> Clone for LinkedListIndex<Element> {
    fn clone(&self) -> Self {
        match self {
            LinkedListIndex::Cursor(cursor) => LinkedListIndex::Cursor(*cursor),
            LinkedListIndex::AfterLast => LinkedListIndex::AfterLast
        }
    }
}

impl<Element> Copy for LinkedListIndex<Element> {}

impl<Element> PartialEq for LinkedListIndex<Element> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (LinkedListIndex::Cursor(cursor1), LinkedListIndex::Cursor(cursor2)) => cursor1 == cursor2,
            (LinkedListIndex::AfterLast, LinkedListIndex::AfterLast) => true,
            _ => false
        }
    }
}

impl<Element> Eq for LinkedListIndex<Element> {}

// linked list is quick-sortable
impl<Element> QuickSortableContainer<Element> for MyLinkedList<Element> {
    type Index = LinkedListIndex<Element>;
    
    fn swap(&mut self, a: Self::Index, b: Self::Index) {
        if let (
            LinkedListIndex::Cursor(a),
            LinkedListIndex::Cursor(b)
        ) = (a, b) {
            self.swap(a, b);
        } else {
            panic!("Cannot swap with AfterLast index")
        }
    }
    
    fn get(&self, index: Self::Index) -> &Element {
        if let LinkedListIndex::Cursor(cursor) = index {
            self.borrow(cursor)
        } else {
            panic!("Cannot get AfterLast index")
        }
    }
    
    fn next_index(&self, index: Self::Index) -> Self::Index {
        if let LinkedListIndex::Cursor(cursor) = index {
            if let Some(next_cursor) = self.next_cursor(cursor) {
                LinkedListIndex::Cursor(next_cursor)
            } else {
                LinkedListIndex::AfterLast
            }
        } else {
            panic!("Cannot get next index of AfterLast index")
        }
    }
    
    fn prev_index(&self, index: Self::Index) -> Self::Index {
        if let LinkedListIndex::Cursor(cursor) = index {
            if let Some(prev_cursor) = self.prev_cursor(cursor) {
                LinkedListIndex::Cursor(prev_cursor)
            } else {
                LinkedListIndex::AfterLast
            }
        } else {
            // it's AfterLast, its previous is the last element
            LinkedListIndex::Cursor(self.end().unwrap())
        }
    }
    
    fn select_pivot_index<
        Comparator: Fn(&Element, &Element) -> Ordering
    >(
        &self, range_begin: Self::Index, _range_end_exclusive: Self::Index,
        _comparator: &Comparator,
    ) -> Self::Index {
        // just use the first element as pivot
        // it's slow to get the middle element of a range in linked list
        return range_begin;
    }
}

//noinspection DuplicatedCode
#[cfg(test)]
mod tests {
    use rand::{Rng, SeedableRng};
    use rand::prelude::StdRng;
    
    use super::*;
    
    #[test]
    fn test_container_agnostic_quick_sort() {
        let mut rng = create_rng();
        
        for _i in 0..1000 {
            let mut vec = random_vec(&mut rng);
            let mut vec_ref = vec.clone();
            
            let slice: &mut [i32] = vec.as_mut_slice();
            let len = slice.len();
            
            container_agnostic_quick_sort(
                slice, &|a, b| a.cmp(b), 0, len, len,
            );
            
            vec_ref.sort();
            
            assert_eq!(vec, vec_ref);
        }
        
        for _i in 0..1000 {
            let mut vec_ref = random_vec(&mut rng);
            let mut list = to_linked_list(&vec_ref);
            
            let begin_cursor = list.begin().unwrap();
            let len = list.size();
            container_agnostic_quick_sort(
                &mut list, &|a, b| a.cmp(b),
                LinkedListIndex::Cursor(begin_cursor), LinkedListIndex::AfterLast, len,
            );
            
            vec_ref.sort();
            
            let list_converted_to_vec: Vec<i32> = list.iter().map(|r| *r).collect();
            
            assert_eq!(list_converted_to_vec, vec_ref);
        }
    }
    
    fn create_rng() -> StdRng {
        let seed: [u8; 32] = [
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
            25, 26, 27, 28, 29, 30, 31, 32,
        ];
        
        let rng: StdRng = SeedableRng::from_seed(seed);
        rng
    }
    
    fn random_vec(rng: &mut StdRng) -> Vec<i32> {
        let size = rng.gen_range(3..2000);
        let max = rng.gen_range(1..500);
        (0..size).map(|_| rng.gen_range(0..max)).collect()
    }
    
    fn to_linked_list(vec: &Vec<i32>) -> MyLinkedList<i32> {
        let mut list = MyLinkedList::new();
        for &x in vec {
            list.push_back(x);
        }
        list
    }
}
