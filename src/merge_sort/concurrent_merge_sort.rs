use std::{alloc, ptr, slice};
use std::alloc::Layout;
use std::cmp::Ordering;
use std::ops::Range;

use crate::merge_sort::merge::merge_multiple_sorted_sequences_smart;
use crate::merge_sort::simple_merge_sort::simple_merge_sort_inplace;

// represents a partition of a range or sub-range.
pub struct RangePartition {
    // if it partitions into n parts, there will be n+1 separators.
    // the first separator is start index (inclusive), the last separator is end index (exclusive).
    // these parts will be endpoints[0]..endpoints[1], endpoints[1]..endpoints[2], ..., endpoints[n-1]..endpoints[n]
    // the total start point is endpoints[0], the total end point is endpoints[n]
    // subpart i is endpoints[i]..endpoints[i+1]
    endpoints: Vec<usize>,
}

impl RangePartition {
    pub fn from_endpoints(endpoints: Vec<usize>) -> RangePartition {
        assert!(endpoints.len() >= 2, "Must have at least one part");
        
        // must be in ascending order
        for i in 1..endpoints.len() {
            assert!(endpoints[i] >= endpoints[i - 1]);
        }
        
        RangePartition { endpoints }
    }
    
    pub fn from_part_sizes(part_sizes: &[usize], start_index: usize) -> RangePartition {
        let mut endpoints = Vec::with_capacity(part_sizes.len() + 1);
        endpoints.push(start_index);
        
        let mut start = start_index;
        for &size in part_sizes {
            start += size;
            endpoints.push(start);
        }
        
        RangePartition { endpoints }
    }
    
    pub fn part_num(&self) -> usize {
        self.endpoints.len() - 1
    }
    
    pub fn part_start(&self, part_index: usize) -> usize {
        self.endpoints[part_index]
    }
    
    pub fn part_end_exclusive(&self, part_index: usize) -> usize {
        self.endpoints[part_index + 1]
    }
    
    pub fn part_at(&self, part_index: usize) -> Range<usize> {
        self.part_start(part_index)..self.part_end_exclusive(part_index)
    }
    
    pub fn part_length(&self, part_index: usize) -> usize {
        self.part_at(part_index).len()
    }
    
    pub fn total_start_index(&self) -> usize {
        self.part_start(0)
    }
    
    pub fn total_end_index_exclusive(&self) -> usize {
        self.part_end_exclusive(self.part_num() - 1)
    }
    
    pub fn total_range(&self) -> Range<usize> {
        self.total_start_index()..self.total_end_index_exclusive()
    }
    
    pub fn evenly_partition(r: Range<usize>, part_num: usize) -> RangePartition {
        let part_len = (r.end - r.start) / part_num; // it will round down
        let mut endpoints = Vec::with_capacity(part_num + 1);
        let mut start = r.start;
        
        for _ in 0..part_num {
            endpoints.push(start as usize);
            start += part_len;
        }
        
        endpoints.push(r.end as usize);
        // if the size cannot be divided evenly, the last part will be larger
        
        RangePartition::from_endpoints(endpoints)
    }
    
    pub fn split_borrow<'a, T>(&self, slice: &'a mut [T]) -> Vec<&'a mut [T]> {
        let mut result: Vec<&mut [T]> = Vec::with_capacity(self.part_num());
        let mut remaining = slice;
        
        for i in 0..self.part_num() {
            let (part, rest) = remaining.split_at_mut(self.part_length(i));
            result.push(part);
            remaining = rest;
        }
        
        result
    }
    
    // given n pivots, separate the range into n+1 parts.
    // the array in the given range is sorted. the pivots are in order.
    // the elements at part i < pivot[i], the elements at part i+1 >= pivot[i]
    fn find_partition_by_pivots<Element, Comparator>(
        arr: &[Element], range: Range<usize>, compare: &Comparator, pivots: &[&Element],
    ) -> RangePartition
        where
            Comparator: Fn(&Element, &Element) -> Ordering
    {
        // n pivots means n+1 parts and n+2 separators
        let mut endpoints: Vec<usize> = Vec::with_capacity(pivots.len() + 2);
        
        endpoints.push(range.start);
        
        let mut curr_searching_range: Range<usize> = range.clone();
        
        for pivot in pivots {
            let pivot_pos = binary_search_leftmost(&arr[curr_searching_range.clone()], compare, pivot);
            endpoints.push(curr_searching_range.start + pivot_pos);
            curr_searching_range.start += pivot_pos;
        }
        
        endpoints.push(range.end);
        
        RangePartition::from_endpoints(endpoints)
    }
}


// concurrent merge sort.
// denote M as the parallelism.
// steps:
// - separates the input array into M parts, then let each thread merge sort its own part concurrently.
// - select M-1 pivot elements, use binary search to find the insertion index of each pivot element in each part, 
//   separating each part into subparts. let's denote P[a][b] as the b-th subpart of the a-th part.
// - concurrently copy each subpart P[..][k] into a temporary buffer for thread k, into temp[k], 
//   each subpart in temp[k], denoted as temp[k][i], comes from P[i][k].
//   we can now compute the size of each temp[k] buffer, thus know where they will merge into the input array.
// - concurrently merge subparts in temp[k] into the input array.
// Space complexity: O(n) where n is input array size.
// Average time complexity:
// - individual merge sort phase: O( (n / M) log(n / M) )
// - pivot selection phase: O( M * M * log(n/M) )
// - copy to temp buffers phase: O( n / M )
// - final M-way merge: O( (n / M) * log M )
// M is much smaller than n, the overall average time complexity is O( (n / M) log (n / M) ).
fn concurrent_merge_sort<Element, Comparator>(
    arr: &mut [Element], compare: &Comparator,
    parallelism: usize,
)
    where
        Element: Send + Sync,
        Comparator: Fn(&Element, &Element) -> Ordering + Send + Sync
{
    assert!(parallelism > 0);
    
    let len: usize = arr.len();
    if len <= 1 {
        return;
    }
    
    if parallelism == 1 || len <= parallelism * 200 {
        simple_merge_sort_inplace(arr, compare);
        return;
    }
    
    let outer_partition = RangePartition::evenly_partition(0..len, parallelism);
    
    // sort each part in each thread concurrently
    crossbeam::thread::scope(|s| {
        let parts = outer_partition.split_borrow(arr);
        
        for part in parts {
            s.spawn(move |_| {
                simple_merge_sort_inplace(part, compare);
            });
        }
    }).unwrap();
    
    // select the pivots from the first part
    let first_part = &arr[outer_partition.part_at(0)];
    let first_part_pivot_partitions: RangePartition = RangePartition::evenly_partition(0..first_part.len(), parallelism);
    let mut pivots: Vec<&Element> = Vec::with_capacity(parallelism - 1);
    for i in 0..parallelism - 1 {
        pivots.push(&first_part[first_part_pivot_partitions.endpoints[i + 1]]);
    }
    
    // sub_partitions[i][j] is the j-th subpart of the i-th part,
    // in the first phase sorted by thread i, in the last phase merged by thread j.
    let mut sub_partitions: Vec<RangePartition> = Vec::with_capacity(parallelism);
    sub_partitions.push(first_part_pivot_partitions);
    
    for thread_index in 1..parallelism {
        let part_partition = RangePartition::find_partition_by_pivots(
            arr, outer_partition.part_at(thread_index), compare, pivots.as_slice(),
        );
        sub_partitions.push(part_partition);
    }
    
    // in the first stage, thread k sorts outer_partition[k], which is sub_partitions[k][..]
    // in the final stage, thread k will merge subpart_partitions[..][k] into the input array.
    
    // the layout in each temp buffer locally
    // temp_partitions[k][i] is the i-th subpart of thread k's temp buffer
    let temp_partitions: Vec<RangePartition> =
        (0..parallelism)
            .map(|thread_index|
                RangePartition::from_part_sizes(
                    (0..parallelism)
                        .map(|subpart_index|
                            sub_partitions[subpart_index].part_length(thread_index)
                        )
                        .collect::<Vec<usize>>().as_slice(),
                    0,
                )
            ).collect::<Vec<RangePartition>>();
    
    // the merging destination layout in the input array
    // result_partitions[k] is the destination of merging of thread k
    let result_partitions: RangePartition = RangePartition::from_part_sizes(
        temp_partitions.iter().map(|p| p.total_range().len()).collect::<Vec<usize>>().as_slice(),
        0,
    );
    
    // do a parallel copy from arr to allocated per-thread temporary buffers.
    // for thread k, it copies sub_partitions[i][k] to temp_partitions[k][i]
    let arr_ptr = SendablePtrWrapper::new(arr.as_mut_ptr());
    let sub_partitions_ref = &sub_partitions; // this should not be inlined
    let temp_partitions_ref = &temp_partitions; // this should not be inlined
    let temps: Vec<*mut Element> = crossbeam::thread::scope(|s| {
        let mut handles = Vec::with_capacity(parallelism);
        
        for thread_index in 0..parallelism {
            let handle = s.spawn(move |_| {
                let alloc_layout = Layout::array::<Element>(
                    temp_partitions_ref[thread_index].total_range().len()
                ).unwrap();
                
                let temp_ptr = unsafe { alloc::alloc(alloc_layout) as *mut Element };
                
                for subpart_index in 0..parallelism {
                    let src = unsafe {
                        arr_ptr.as_mut_ptr().add(sub_partitions_ref[subpart_index].part_start(thread_index))
                    };
                    let dst = unsafe {
                        temp_ptr.add(temp_partitions_ref[thread_index].part_start(subpart_index))
                    };
                    let len = sub_partitions_ref[subpart_index].part_length(thread_index);
                    unsafe {
                        ptr::copy_nonoverlapping(src, dst, len);
                    }
                }
                
                return SendablePtrWrapper::new(temp_ptr);
            });
            
            handles.push(handle);
        }
        
        handles.into_iter().map(
            |handle| {
                let wrapper: SendablePtrWrapper<Element> = handle.join().unwrap();
                wrapper.as_mut_ptr()
            }
        ).collect()
    }).unwrap();
    
    // for thread k, do a multi-way merge for the k-th subpart of each part
    // from temps[k] to arr[result_partitions[k-1]..result_partitions[k]]
    
    crossbeam::thread::scope(|s| {
        for thread_index in 0..parallelism {
            let merge_srcs: Vec<&[Element]> = (0..parallelism).map(
                |subpart_index| {
                    unsafe {
                        let range = temp_partitions[thread_index].part_at(subpart_index);
                        slice::from_raw_parts(
                            temps[thread_index].add(range.start), range.len(),
                        )
                    }
                }
            ).collect();
            
            let merge_dst: SendablePtrWrapper<Element> = unsafe {
                SendablePtrWrapper::new(
                    arr.as_mut_ptr().add(result_partitions.part_start(thread_index))
                )
            };
            let merge_region_len = result_partitions.part_length(thread_index);
            
            s.spawn(move |_| {
                merge_multiple_sorted_sequences_smart(
                    merge_srcs.as_slice(), compare,
                    &mut |index, element| {
                        unsafe {
                            assert!(index < merge_region_len);
                            ptr::write(merge_dst.as_mut_ptr().add(index), ptr::read(element));
                        }
                    },
                );
            });
        }
    }).unwrap();
    
    // free temp buffers
    for i in 0..parallelism {
        let layout = Layout::array::<Element>(temp_partitions[i].total_range().len()).unwrap();
        unsafe {
            alloc::dealloc(temps[i] as *mut u8, layout);
        }
    }
}

// when binary_search in std found consecutive equal elements, it may not return the leftmost one.
// this function will return the leftmost one.
fn binary_search_leftmost<Element, Comparator>(
    arr: &[Element], compare: &Comparator, target: &Element,
) -> usize
    where
        Comparator: Fn(&Element, &Element) -> Ordering
{
    match arr.binary_search_by(|probe| compare(probe, target)) {
        Ok(pos) => {
            // if there are no more element on the left, it's the leftmost
            if pos == 0 {
                return pos;
            }
            
            // if the left element is not equal, it's the leftmost
            if compare(&arr[pos - 1], target) != Ordering::Equal {
                return pos;
            }
            
            // the left element exists and is equal. 
            // there may be many equal elements on the left,
            // so use a recursive binary search, instead of a linear search.
            return binary_search_leftmost(&arr[0..pos], compare, target);
        }
        // equal element not found, return the insertion index
        Err(pos) => pos
    }
}

// in Rust, mut pointer is not Send or Sync by default, so create this wrapper to workaround it.
pub struct SendablePtrWrapper<T> {
    ptr: *mut T,
}

impl<T> SendablePtrWrapper<T> {
    pub fn new(ptr: *mut T) -> SendablePtrWrapper<T> {
        SendablePtrWrapper { ptr }
    }
    
    // should use this instead of directly accessing field.
    // when accessing field, Rust will consider it as borrowing the field, thus still not sendable.
    pub fn as_mut_ptr(&self) -> *mut T {
        self.ptr
    }
}

unsafe impl<T> Send for SendablePtrWrapper<T> {}

unsafe impl<T> Sync for SendablePtrWrapper<T> {}

impl<T> Clone for SendablePtrWrapper<T> {
    fn clone(&self) -> Self {
        SendablePtrWrapper { ptr: self.ptr }
    }
}

impl<T> Copy for SendablePtrWrapper<T> {}

#[cfg(test)]
mod tests {
    use std::time::Instant;
    
    use rand::{Rng, SeedableRng};
    use rand::prelude::StdRng;
    
    use super::*;
    
    #[test]
    fn test_binary_search_leftmost() {
        let arr = [1, 2, 2, 2, 3, 4, 5, 6, 7, 8, 9];
        
        assert_eq!(binary_search_leftmost(&arr, &|a, b| a.cmp(b), &2), 1);
        assert_eq!(binary_search_leftmost(&arr, &|a, b| a.cmp(b), &3), 4);
        assert_eq!(binary_search_leftmost(&arr, &|a, b| a.cmp(b), &9), 10);
        assert_eq!(binary_search_leftmost(&arr, &|a, b| a.cmp(b), &0), 0);
        assert_eq!(binary_search_leftmost(&arr, &|a, b| a.cmp(b), &10), 11);
    }
    
    #[test]
    fn test_concurrent_merge_sort() {
        let mut rng: StdRng = SeedableRng::seed_from_u64(123456);
        
        // sort integer
        for _i in 0..100 {
            let len = rng.gen_range(0..100000);
            let max = rng.gen_range(0..10000);
            let parallelism = rng.gen_range(1..16);
            
            let mut arr: Vec<i32> = (0..len).map(|_| rng.gen_range(0..max)).collect();
            let mut arr_for_ref = arr.clone();
            
            concurrent_merge_sort(&mut arr, &|a, b| a.cmp(b), parallelism);
            arr_for_ref.sort();
            
            assert_eq!(arr, arr_for_ref);
        }
        
        // sort string by length, testing stability
        for _i in 0..100 {
            let len = rng.gen_range(0..10000);
            let parallelism = rng.gen_range(1..16);
            
            let mut arr: Vec<String> = (0..len).map(|_| {
                let len = rng.gen_range(1..10);
                (0..len).map(|_| rng.gen_range(('a' as u8)..=('z' as u8)) as char).collect()
            }).collect();
            let mut arr_for_ref = arr.clone();
            
            concurrent_merge_sort(&mut arr, &|a, b| a.len().cmp(&b.len()), parallelism);
            arr_for_ref.sort_by(|a, b| a.len().cmp(&b.len()));
            
            assert_eq!(arr, arr_for_ref);
        };
    }
    
    #[test]
    #[ignore]
    fn test_concurrent_merge_sort_time() {
        let mut rng: StdRng = SeedableRng::seed_from_u64(123456);
        
        let len = 4000000;
        let max = 100000000;
        
        let arr: Vec<i32> = (0..len).map(|_| rng.gen_range(0..max)).collect();
        
        test_time_for(&arr, 1);
        test_time_for(&arr, 2);
        test_time_for(&arr, 4);
        test_time_for(&arr, 8);
        test_time_for(&arr, 16);
        test_time_for(&arr, 32);
        
        // test again
        test_time_for(&arr, 1);
        test_time_for(&arr, 2);
        test_time_for(&arr, 4);
        test_time_for(&arr, 8);
        test_time_for(&arr, 16);
        test_time_for(&arr, 32);
    }
    
    fn test_time_for(mut arr: &Vec<i32>, parallelism: usize) {
        let mut to_sort = arr.clone();
        let start = Instant::now();
        concurrent_merge_sort(&mut to_sort, &|a, b| a.cmp(b), parallelism);
        let duration = start.elapsed();
        
        println!("concurrent_merge_sort parallelism {:?} time: {:?}", parallelism, duration);
    }
}