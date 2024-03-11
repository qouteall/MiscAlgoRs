pub struct LazyQuickSorter<'a, Element, SortKey, SortKeyExtractor>
where
    SortKey: Ord,
    SortKeyExtractor: Fn(&Element) -> SortKey,
{
    arr: &'a mut [Element],
    extract_sort_key: SortKeyExtractor,
    root_node: LazyQuickSorterNodeState,
}

// Initially, the range is unsorted. Each search for an element will ensure that the path from root to the target child is sorted.
// When two childs are sorted, the parent is marked as sorted.
enum LazyQuickSorterNodeState {
    Unsorted,
    PartiallySorted(Box<PartialSortData>),
    FullySorted,
}

struct PartialSortData {
    pivot_index: usize,
    left_child: LazyQuickSorterNodeState,
    right_child: LazyQuickSorterNodeState,
}

impl<'a, Element, SortKey, SortKeyExtractor> LazyQuickSorter<'a, Element, SortKey, SortKeyExtractor>
where
    SortKey: Ord,
    SortKeyExtractor: Fn(&Element) -> SortKey,
{
    pub fn new(
        arr: &'a mut [Element],
        extract_sort_key: SortKeyExtractor,
    ) -> LazyQuickSorter<'a, Element, SortKey, SortKeyExtractor> {
        LazyQuickSorter {
            arr,
            extract_sort_key,
            root_node: LazyQuickSorterNodeState::Unsorted,
        }
    }

    pub fn at(&mut self, index: usize) -> &Element {
        LazyQuickSorter::ensure_sorted(
            &mut self.root_node,
            index,
            0,
            self.arr.len(),
            self.arr,
            &self.extract_sort_key,
        );
        return &self.arr[index];
    }

    fn ensure_sorted(
        node: &mut LazyQuickSorterNodeState,
        target_index: usize,
        range_left: usize,
        range_right_exclusive: usize,
        arr: &mut [Element],
        extract_sort_key: &SortKeyExtractor,
    ) {
        if range_right_exclusive - range_left <= 1{
            *node = LazyQuickSorterNodeState::FullySorted;
            return;
        }

        match *node {
            LazyQuickSorterNodeState::Unsorted => {
                let pivot_index = range_left + (range_right_exclusive - range_left) / 2;
                LazyQuickSorter::partition_around_pivot(
                    pivot_index,
                    range_left,
                    range_right_exclusive,
                    arr,
                    extract_sort_key,
                );
                let mut partial_sort_data = PartialSortData {
                    pivot_index: pivot_index,
                    left_child: LazyQuickSorterNodeState::Unsorted,
                    right_child: LazyQuickSorterNodeState::Unsorted,
                };

                if target_index <= pivot_index {
                    LazyQuickSorter::ensure_sorted(
                        &mut partial_sort_data.left_child,
                        target_index,
                        range_left,
                        pivot_index,
                        arr,
                        extract_sort_key,
                    );
                } else {
                    LazyQuickSorter::ensure_sorted(
                        &mut partial_sort_data.right_child,
                        target_index,
                        pivot_index + 1,
                        range_right_exclusive,
                        arr,
                        extract_sort_key,
                    );
                }

                if let (
                    LazyQuickSorterNodeState::FullySorted,
                    LazyQuickSorterNodeState::FullySorted,
                ) = (
                    &partial_sort_data.left_child,
                    &partial_sort_data.right_child,
                ) {
                    *node = LazyQuickSorterNodeState::FullySorted;
                } else {
                    *node = LazyQuickSorterNodeState::PartiallySorted(Box::new(partial_sort_data));
                }
            }
            LazyQuickSorterNodeState::PartiallySorted(ref mut partial_sort_data) => {
                let pivot_index = partial_sort_data.pivot_index;
                if target_index <= pivot_index {
                    LazyQuickSorter::ensure_sorted(
                        &mut partial_sort_data.left_child,
                        target_index,
                        range_left,
                        pivot_index,
                        arr,
                        extract_sort_key,
                    );
                } else {
                    LazyQuickSorter::ensure_sorted(
                        &mut partial_sort_data.right_child,
                        target_index,
                        pivot_index + 1,
                        range_right_exclusive,
                        arr,
                        extract_sort_key,
                    );
                }

                if let (
                    LazyQuickSorterNodeState::FullySorted,
                    LazyQuickSorterNodeState::FullySorted,
                ) = (
                    &partial_sort_data.left_child,
                    &partial_sort_data.right_child,
                ) {
                    *node = LazyQuickSorterNodeState::FullySorted;
                }
            }
            LazyQuickSorterNodeState::FullySorted => {}
        }
    }

    fn partition_around_pivot(
        pivot_index: usize,
        range_left: usize,
        range_right_exclusive: usize,
        arr: &mut [Element],
        extract_sort_key: &SortKeyExtractor,
    ) {
        let mut left_index = range_left;
        let mut right_index = range_right_exclusive - 1;

        while left_index < right_index {
            while extract_sort_key(&arr[left_index]) < extract_sort_key(&arr[pivot_index]) {
                left_index += 1;
            }
            while extract_sort_key(&arr[right_index]) > extract_sort_key(&arr[pivot_index]) {
                right_index -= 1;
            }
            if left_index <= right_index {
                arr.swap(left_index, right_index);
                left_index += 1;
                right_index -= 1;
            }
        }
    }
}
