use std::cmp::Ordering;

use crate::quick_sort::partition::fat_partition_no_clone_required;
use crate::quick_sort::pivot_select::median_of_three_pivot;

// Quick Sort is a divide-and-conquer algorithm.
// In each step, it separates the list into the left part and the right part, where left part <= pivot and right part >= pivot.
// The elements in each part will be sorted by recursively.
// The partition process is minimizing the freedom of the position of each elements.
// In average case, in each partition process, each element's possible position range is cut by half.
// In the end, each element's position freedom is reduced to minimal, having the list sorted.
// If we only want the n-th element in an array, we don't need to sort the whole array,
// we only need to sort the parts that contains the n-th element.
// Because of the separation, not sorting the unrelated parts will not affect the position.
pub struct LazyQuickSorter<'a, Element, Comparator>
    where
        Comparator: Fn(&Element, &Element) -> Ordering,
{
    arr: &'a mut [Element],
    comparator: &'a Comparator,
    root_node: NodeState,
}

// The partitioning process creates a binary tree, where each node corresponds to a range in the array.
// If the node is fully sorted or fully unsorted, the node is a leaf node.
// When beginning sorting a fully unsorted range, create new node marking partially sorted.
// When finishing sorting a range, mark the node as fully sorted.
enum NodeState {
    // Not yet partitioned.
    Unsorted,
    
    // This layer is partitioned.
    PartiallySorted(Box<PartialSortData>),
    
    // Fully sorted.
    FullySorted,
}

struct PartialSortData {
    // by using fat partition,
    // the left part is [range_left..partition_left] (does not include partition_left)
    // the right part is [partition_right..range_right_exclusive] (includes partition_right)
    partition_left: usize,
    partition_right: usize,
    left_child: NodeState,
    right_child: NodeState,
}

impl<'a, Element, Comparator> LazyQuickSorter<'a, Element, Comparator>
    where
        Comparator: Fn(&Element, &Element) -> Ordering
{
    pub fn new(
        arr: &'a mut [Element],
        comparator: &'a Comparator,
    ) -> LazyQuickSorter<'a, Element, Comparator> {
        LazyQuickSorter {
            arr,
            comparator,
            root_node: NodeState::Unsorted,
        }
    }
    
    // Get the index + 1 th smallest element in the array.
    // Time complexity is O(log n) where n is list size.
    pub fn at(&mut self, index: usize) -> &Element {
        LazyQuickSorter::ensure_sorted(
            &mut self.root_node,
            index,
            0,
            self.arr.len(),
            self.arr,
            self.comparator,
        );
        return &self.arr[index];
    }
    
    // Ensure that the element at the target_index is sorted, in the context of a range.
    // Each range correspond to a node in the tree.
    fn ensure_sorted(
        node: &mut NodeState,
        target_index: usize,
        range_left: usize,
        range_right_exclusive: usize,
        arr: &mut [Element],
        comparator: &Comparator,
    ) {
        let len = range_right_exclusive - range_left;
        
        assert!(len > 0);
        
        if len == 1 {
            *node = NodeState::FullySorted;
            return;
        }
        
        if len == 2 {
            match *node {
                NodeState::Unsorted => {}
                NodeState::PartiallySorted(ref mut _partial_sort_data) => {}
                NodeState::FullySorted => {
                    return;
                }
            }
            
            if (comparator)(&arr[range_left], &arr[range_left + 1]) == Ordering::Greater {
                arr.swap(range_left, range_left + 1);
            }
            
            *node = NodeState::FullySorted;
            return;
        }
        
        match *node {
            NodeState::Unsorted => {
                // The range is unsorted.
                // We need to partition the range around a pivot,
                // and then recursively lazily sort a subrange if necessary.
                
                let pivot_index = median_of_three_pivot(
                    &mut arr[range_left..range_right_exclusive], comparator,
                );
                
                // pl and pr are relative to the subslice
                let (pl, pr) = fat_partition_no_clone_required(
                    arr[range_left..range_right_exclusive].as_mut(),
                    comparator, pivot_index,
                );
                let partition_left = range_left + pl;
                let partition_right = range_left + pr;
                
                let mut partial_sort_data = PartialSortData {
                    partition_left,
                    partition_right,
                    left_child: NodeState::Unsorted,
                    right_child: NodeState::Unsorted,
                };
                
                if target_index < partition_left {
                    // Sort the left child range.
                    LazyQuickSorter::ensure_sorted(
                        &mut partial_sort_data.left_child,
                        target_index,
                        range_left,
                        partition_left,
                        arr,
                        comparator,
                    );
                } else if target_index >= partition_right {
                    // Sort the right child range.
                    LazyQuickSorter::ensure_sorted(
                        &mut partial_sort_data.right_child,
                        target_index,
                        partition_right,
                        range_right_exclusive,
                        arr,
                        comparator,
                    );
                } else {
                    // The target_index is in the "equal" region
                    // no need to do recursive sorting
                }
                
                // Mark the node as partially sorted.
                *node = NodeState::PartiallySorted(Box::new(partial_sort_data));
            }
            NodeState::PartiallySorted(ref mut partial_sort_data) => {
                // partial_sort_data: &mut Box<PartialSortData>
                if target_index < partial_sort_data.partition_left {
                    // Sort the left child range.
                    LazyQuickSorter::ensure_sorted(
                        &mut partial_sort_data.left_child,
                        target_index,
                        range_left,
                        partial_sort_data.partition_left,
                        arr,
                        comparator,
                    );
                } else if target_index >= partial_sort_data.partition_right {
                    // Sort the right child range.
                    LazyQuickSorter::ensure_sorted(
                        &mut partial_sort_data.right_child,
                        target_index,
                        partial_sort_data.partition_right,
                        range_right_exclusive,
                        arr,
                        comparator,
                    );
                } else {
                    // The target_index is in the "equal" region
                    // no need to do recursive sorting
                    return;
                }
                
                // If both childs are sorted, mark the node as fully sorted.
                if let (NodeState::FullySorted, NodeState::FullySorted) = (
                    &partial_sort_data.left_child,
                    &partial_sort_data.right_child,
                ) {
                    *node = NodeState::FullySorted;
                }
            }
            NodeState::FullySorted => {
                // The range is already sorted. Do nothing.
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use rand::{Rng, rngs::StdRng, SeedableRng};
    
    use super::*;
    
    #[test]
    #[ignore]
    fn simple_test_lazy_quick_sort() {
        let mut arr = [7, 4, 399, 1, 99, -3];
        
        let mut s = LazyQuickSorter::new(&mut arr, &|x: &i32, y: &i32| x.cmp(y));
        
        println!("{:?}", s.at(0));
        println!("{:?}", s.at(1));
        println!("{:?}", s.at(2));
        println!("{:?}", s.at(3));
        println!("{:?}", s.at(4));
        println!("{:?}", s.at(5));
    }
    
    #[test]
    fn test_lazy_quick_sort() {
        let mut rng = create_rng();
        
        let size = rng.gen_range(0..1000);
        let vec: Vec<i32> = (0..size).map(|_| rng.gen_range(0..2000)).collect();
        
        test_lazy_quick_sort_for(&vec, &mut rng);
    }
    
    fn create_rng() -> StdRng {
        let seed: [u8; 32] = [
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
            25, 26, 27, 28, 29, 30, 31, 32,
        ];
        
        let rng: StdRng = SeedableRng::from_seed(seed);
        rng
    }
    
    fn test_lazy_quick_sort_for(vec: &Vec<i32>, rng: &mut StdRng) {
        let mut vec_copy1 = vec.clone();
        let mut vec_copy2 = vec.clone();
        
        let slice: &mut [i32] = vec_copy1.as_mut_slice();
        
        let mut s = LazyQuickSorter::new(slice, &|x: &i32, y: &i32| x.cmp(y));
        
        vec_copy2.sort();
        
        for _i in 0..3000 {
            let index = rng.gen_range(0..vec_copy2.len());
            assert_eq!(*s.at(index), vec_copy2[index]);
        }
    }
}
