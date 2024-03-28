use std::alloc::Layout;
use std::cmp::Ordering;
use std::cmp::Ordering::Less;
use std::ptr;
use std::slice::from_raw_parts_mut;

use crate::data_structure::binary_heap::MyMinHeap;

pub fn merge_two_sorted_sequences<Element, Comparator, ResultConsumer>(
    arr1: &[Element], arr2: &[Element],
    compare: &Comparator,
    result_consumer: &mut ResultConsumer,
)
    where Comparator: Fn(&Element, &Element) -> Ordering,
          ResultConsumer: FnMut(usize, &Element)
{
    let mut i1 = 0;
    let mut i2 = 0;
    
    while i1 < arr1.len() && i2 < arr2.len() {
        let ordering = compare(&arr1[i1], &arr2[i2]);
        match ordering {
            Ordering::Less => {
                result_consumer(i1 + i2, &arr1[i1]);
                i1 += 1;
            }
            Ordering::Equal => {
                // output i1 first
                result_consumer(i1 + i2, &arr1[i1]);
                i1 += 1;
                // we should not output arr2[i2] here, 
                // because there may be another element in arr1 that is equal to arr2[i2], but should be output before arr2[i2].
            }
            Ordering::Greater => {
                result_consumer(i1 + i2, &arr2[i2]);
                i2 += 1;
            }
        }
    }
    
    while i1 < arr1.len() {
        result_consumer(i1 + i2, &arr1[i1]);
        i1 += 1;
    }
    
    while i2 < arr2.len() {
        result_consumer(i1 + i2, &arr2[i2]);
        i2 += 1;
    }
}

// it merges multiple sorted sequences into one sorted sequence,
// by continuously selecting the minimum element from the heads of the sequences.
pub fn merge_multiple_sorted_sequences_naive<Element, Comparator, ResultConsumer>(
    arrs: &[&[Element]],
    compare: &Comparator,
    
    // it takes an output index and an element reference
    result_consumer: &mut ResultConsumer,
)
    where Comparator: Fn(&Element, &Element) -> Ordering,
          ResultConsumer: FnMut(usize, &Element)
{
    assert!(arrs.len() >= 2);
    
    // indices[i] is the index of the next element to check from arrs[i]
    let mut indices: Vec<usize> = vec![0; arrs.len()];
    
    // the index of the next element to output
    let mut placing_index = 0;
    
    loop {
        // find the minimum element in arrays
        
        // (curr_min_arr_index, &curr_min_element)
        let mut min_found: Option<(usize, &Element)> = None;
        
        for arr_index in 0..arrs.len() {
            let index_in_arr = indices[arr_index];
            let curr_arr = arrs[arr_index];
            if index_in_arr < curr_arr.len() {
                let element: &Element = &curr_arr[index_in_arr];
                if let Some((_curr_min_arr_index, curr_min_element)) = min_found {
                    // compare with the current min element
                    // if it equals the current min element, still not replace the found min, to make the sort stable
                    if compare(element, curr_min_element) == Less {
                        min_found = Some((arr_index, element))
                    }
                } else {
                    // no element scanned yet. This is the first one.
                    min_found = Some((arr_index, element))
                }
            }
        }
        
        match min_found {
            None => {
                // no min found means all arrays are exhausted
                return;
            }
            Some((min_arr_index, min_element)) => {
                // output it
                result_consumer(placing_index, min_element);
                indices[min_arr_index] += 1;
                placing_index += 1;
            }
        }
    }
}

struct MinHeapElement<'a, Element> {
    element: &'a Element,
    arr_index: usize,
}

// It merges multiple sorted sequences into one sorted sequence,
// by continuously selecting the minimum element using a min heap.
pub fn merge_multiple_sorted_sequences_smart<Element, Comparator, ResultConsumer>(
    arrs: &[&[Element]],
    comparator: &Comparator,
    
    // it takes an output index and an element reference
    result_consumer: &mut ResultConsumer,
)
    where Comparator: Fn(&Element, &Element) -> Ordering,
          ResultConsumer: FnMut(usize, &Element)
{
    assert!(arrs.len() >= 2);
    
    // indices[i] is the index of the next element to check from arrs[i]
    let mut indices: Vec<usize> = vec![0; arrs.len()];
    
    let heap_comparator = |e1: &MinHeapElement<Element>, e2: &MinHeapElement<Element>| {
        // the min heap is not stable, but we want to make the sort stable
        // if elements are equal, the former array is considered smaller and should be output first
        comparator(&e1.element, &e2.element)
            .then(e1.arr_index.cmp(&e2.arr_index))
    };
    let mut min_heap: MyMinHeap<MinHeapElement<Element>, _> = MyMinHeap::new(&heap_comparator);
    
    // initialize the min heap with the first element of each array
    for (arr_index, arr) in arrs.iter().enumerate() {
        if !arr.is_empty() {
            min_heap.insert(MinHeapElement {
                element: &arr[0],
                arr_index,
            });
            indices[arr_index] = 1;
        }
    }
    
    let mut placing_index = 0;
    
    loop {
        let min = min_heap.take_min();
        
        match min {
            None => {
                // no min found means all arrays are exhausted
                return;
            }
            Some(MinHeapElement { element, arr_index }) => {
                // output it
                result_consumer(placing_index, element);
                placing_index += 1;
                
                let next_index = indices[arr_index];
                if next_index < arrs[arr_index].len() {
                    min_heap.insert(MinHeapElement {
                        element: &arrs[arr_index][next_index],
                        arr_index,
                    });
                    indices[arr_index] = next_index + 1;
                }
            }
        }
    }
}

// It merges two adjacent sorted sequences arr[0..separation_index] and arr[separation_index..], inplace.
// "Smart" means it uses binary search to reduce the range to merge.
pub fn smart_merge_two_adjacent_sorted_sequences_inplace<Element, Comparator>(
    arr: &mut [Element],
    separation_index: usize,
    compare: &Comparator,
)
    where Comparator: Fn(&Element, &Element) -> Ordering
{
    let len = arr.len();
    if len <= 1 {
        return;
    }
    
    if separation_index == 0 || separation_index == len {
        return;
    }
    
    let (left_part, right_part) = arr.split_at_mut(separation_index);
    
    let left_max: &Element = &left_part[separation_index - 1];
    
    let right_min: &Element = &right_part[0];
    
    // if left_max <= right_min, no need to merge
    if compare(left_max, right_min) != Ordering::Greater {
        return;
    }
    
    // arr[right_delim_index..] >= left_max, thus don't need to be processed
    let right_delimit_index = match right_part.binary_search_by(|x| compare(x, left_max)) {
        // right_part[index] == left_max, so right_part[index..] >= left_max,
        // no need to process right_part[index..], 
        // which is arr[separation_index+index..]
        Ok(index) => separation_index + index,
        
        // binary search giving Err means that left_max should be inserted at right_part[index] to maintain the order
        // right_part[..index] < left_max < right_part[index], right_part[index..] > left_max, no need to process right_part[index..]
        // which is arr[separation_index+index..]
        Err(index) => separation_index + index,
    };
    
    // arr[..left_delim_index] <= right_min, thus don't need to be processed
    let left_delimit_index = match left_part.binary_search_by(|x| compare(x, right_min)) {
        // left_part[index] == right_min, so left_part[..=index] <= right_min,
        // no need to process left_part[..=index], 
        // which is arr[..index+1]
        Ok(index) => index + 1,
        
        // binary search giving Err means that right_min should be inserted at left_part[index] to maintain the order
        // left_part[..index] < right_min < left_part[index]
        // left_part[..index] <= right_min, thus don't need to be processed
        // which is arr[..index]
        Err(index) => index,
    };
    
    // now we need to merge arr[left_delimit_index..separation_index] and arr[separation_index..right_delimit_index]
    // into arr[left_delimit_index..right_delimit_index]
    if separation_index == left_delimit_index || separation_index == right_delimit_index {
        return;
    }
    
    merge_two_adjacent_sorted_sequences_inplace(
        &mut arr[left_delimit_index..right_delimit_index],
        separation_index - left_delimit_index,
        compare,
    );
}

// It merges two adjacent sorted sequences arr[0..separation_index] and arr[separation_index..].
// It first copies the left part to a temporary array, then merge the temporary array and the right part into arr.
pub fn merge_two_adjacent_sorted_sequences_inplace<Element, Comparator>(
    arr: &mut [Element],
    separation_index: usize,
    compare: &Comparator,
)
    where Comparator: Fn(&Element, &Element) -> Ordering
{
    let len: usize = arr.len();
    
    assert!(separation_index <= len);
    
    if separation_index == 0 || separation_index == len {
        return;
    }
    
    // for performance, we don't want to clone the element. instead we move it.
    // so the element at the original position will be temporarily invalid.
    // this is not allowed in safe Rust, so we use unsafe.
    
    let alloc_layout: Layout = match Layout::array::<Element>(separation_index) {
        Ok(alloc_layout) => alloc_layout,
        Err(_) => {
            panic!("Unable to allocate memory for merge_two_adjacent_sorted_sequences_inplace");
        }
    };
    
    // allocate memory for the temporary array.
    // it will hold the left part temporarily.
    let temp: *mut Element = unsafe {
        std::alloc::alloc(alloc_layout) as *mut Element
    };
    
    // copy the left part to temp
    unsafe {
        ptr::copy_nonoverlapping(arr.as_ptr(), temp, separation_index);
        // the left part in arr is temporarily in invalid state now.
    }
    
    let temp_slice = unsafe { from_raw_parts_mut(temp, separation_index) };
    
    let arr_ptr = arr.as_mut_ptr();
    
    // merge the temp and right part into arr
    // in the merging process, if it selects an element from tmp, the merged region will grow by one,
    // if it selects an element from the right part, the merged region will also grow, and the right region will shrink by one.
    // in the end, the merged region will cover the whole arr.
    merge_two_sorted_sequences(
        &temp_slice,
        &arr[separation_index..],
        compare,
        &mut |index, element| {
            unsafe {
                assert!(index < len);
                
                ptr::write(arr_ptr.add(index), ptr::read(element));
            }
        },
    );
    
    // free the memory. it will not call drop on the elements in temp.
    unsafe {
        std::alloc::dealloc(temp as *mut u8, alloc_layout);
    }
    
    // Note: the temp buffer may memory leak when panic happens inside comparator.
    // can be fixed by a type that implements Drop.
}