use std::cmp::Ordering;
use std::cmp::Ordering::{Greater, Less};

// Reference: https://en.wikipedia.org/wiki/Quicksort
// This file contains:
// * Lomuto partition. 
//   It's the basic partition that scans from left to right.
// * Hoare partition. 
//   It scans from both sides, resulting in fewer swaps than Lomuto partition.
// * Fat partition (Dutch national flag partition). 
//   It partitions an extra "equal" range that works better than Hoare partition when many equal elements exist.
// The pivot selection is passed as argument. Pivot selection should be separated from partition algorithm.

// Note: if Clone or comparator is implemented wrongly, these algorithms will malfunction.
// The order provided by comparator should be consistent:
// * reflective: a == a
// * transitive: if a < b and b < c, then a < c, if a == b and b == c, then a == c
// * anti-symmetric: if a < b, then b > a
// * order does not rely on interior mutability that can change during sorting
// * order of an element should not change after cloning

// Lomuto partition (requires Clone)
// for return value r, 
// it ensures arr[0..r] < pivot, arr[r] == pivot, and arr[(r+1)..] >= pivot
// Note: the pivot may get moved
pub fn lomuto_partition<Element: Clone, Comparator>(
    arr: &mut [Element],
    comparator: &Comparator,
    pivot_index: usize,
) -> usize
    where
        Comparator: Fn(&Element, &Element) -> Ordering,
        Element: Clone,
{
    let len = arr.len();
    assert!(len > 2);
    
    // move the pivot to the end
    arr.swap(pivot_index, len - 1);
    
    // it clones the pivot element to stack
    let pivot = arr[len - 1].clone();
    
    // we want:
    // arr[0..left_index] < pivot (does not include left_index)
    // arr[left_index..j] >= pivot (does not include j)
    // left_index <= j
    let mut left_index = 0;
    for j in 0..len - 1 {
        // given arr[0..left_index] < pivot, arr[left_index..j] >= pivot (it includes arr[left_index] >= pivot).
        
        // if arr[j] >= pivot, then arr[left_index..j+1] >= pivot, we can substitute j with j+1 .
        
        // if arr[j] < pivot, we already know that arr[left_index] >= pivot,
        // swap arr[left_index] with arr[j], 
        // now arr[j] >= pivot, arr[left_index] < pivot, 
        // then arr[left_index..(j+1)] >= pivot, we can substitute j with j+1 ,
        // and arr[0..(left_index+1)] < pivot, we can substitute left_index with left_index+1
        if comparator(&arr[j], &pivot) == Ordering::Less {
            arr.swap(left_index, j);
            left_index += 1;
        }
    }
    
    // now, j = len - 1
    // arr[0..left_index] < pivot, arr[left_index..(len-1)] >= pivot, arr[len-1] == pivot
    
    // move pivot to the separation point
    arr.swap(left_index, len - 1);
    
    // now:
    // arr[0..left_index] < pivot (does not include left_index)
    // arr[left_index] == pivot
    // arr[left_index+1..len] >= pivot
    
    left_index
}


// Hoare partition (requires Clone)
// for return value p, it ensures arr[0..p] <= pivot and arr[p..] > pivot
// the left part should not be empty: p > 0
// the right part should not be empty: p < arr.len()
// Note: the pivot may get moved
pub fn hoare_partition<Element: Clone, Comparator>(
    arr: &mut [Element],
    comparator: &Comparator,
    pivot_index: usize,
) -> usize
    where
        Comparator: Fn(&Element, &Element) -> Ordering,
        Element: Clone,
{
    let len = arr.len();
    assert!(len > 2, "the array should have at least 3 elements");
    
    // it clones the pivot element to stack
    let pivot = arr[pivot_index].clone();
    
    let mut left_index = 0;
    let mut right_index = len - 1;
    
    // we want:
    // arr[0..left_index] <= pivot (does not include left_index)
    // arr[(right_index + 1)..] > pivot (does not include right_index)
    // left_index >= 0 as it only moves forward, right_index < len as it only moves backward
    loop {
        // given arr[0..left_index] <= pivot,
        // if arr[left_index] <= pivot,
        // then arr[0..left_index+1] <= pivot,
        // then we can substitute left_index with left_index+1
        while comparator(&arr[left_index], &pivot) == Ordering::Less {
            left_index += 1;
        }
        // exiting the loop means arr[left_index] >= pivot
        
        // given arr[(right_index + 1)..] > pivot,
        // if arr[right_index] > pivot,
        // then arr[(right_index - 1 + 1)..] > pivot,
        // then we can substitute right_index with right_index-1
        while comparator(&arr[right_index], &pivot) == Ordering::Greater {
            right_index -= 1;
        }
        // exiting the loop means arr[right_index] <= pivot
        
        if left_index >= right_index {
            // now:
            // arr[0..left_index] <= pivot, arr[(right_index + 1)..] > pivot
            // arr[left_index] >= pivot
            // arr[right_index] <= pivot
            
            if left_index == right_index {
                // if left_index == right_index,
                // pivot <= arr[left_index] <= pivot, so arr[left_index] == pivot
                // arr[0..(left_index+1)] <= pivot, arr[(left_index+1)..] > pivot
                
                // 0 <= left_index == right_index < len
                
                // returning left_index or left_index + 1 both holds the invariant
                // we want it to do actual partition. the left part and right part should not be empty,
                // the partition point should not be 0 (left part empty) or len (right part empty)
                
                if left_index == 0 {
                    // if left_index hasn't moved, it will be 0
                    // in this case, returning left_index will cause left part to be empty,
                    // so return left_index + 1 (it equals 1 now, will not get out of bound)
                    return left_index + 1;
                } else {
                    // if right_index hasn't moved, it will be len - 1
                    // in this case, returning left_index + 1 (it equals right_index + 1) may cause right part to be empty,
                    // so return left_index
                    
                    // 0 < left_index < len, it will not get out of bound
                    
                    return left_index;
                }
            } else if left_index == right_index + 1 {
                // if left_index == right_index + 1
                // arr[0..left_index] <= pivot, arr[left_index..] > pivot
                
                // 0 <= left_index == right_index + 1 < len -1 + 1, it will not get out of bound
                
                return left_index;
            } else {
                // it's impossible that left_index > right_index + 1
                unreachable!();
            }
        }
        // now 0 <= left_index < right_index < len
        
        // we already know that arr[left_index] >= pivot, arr[right_index] <= pivot
        // swap them so that arr[left_index] <= pivot, arr[right_index] >= pivot
        // then we can substitute left_index with left_index+1, right_index with right_index-1
        arr.swap(left_index, right_index);
        
        left_index += 1;
        right_index -= 1;
        // now left_index <= right_index + 1, right_index < len - 1
    }
}

// Fat partition (Dutch national flag partition) (requires Clone)
// It returns (l, r) where arr[0..l] < pivot, arr[l..r] == pivot, and arr[r..] > pivot
// (l is not in the left region but r is in the right region)
// The left region or right region could be empty, but the equal region cannot be empty, as pivot is selected from the array.
// 0 <= l < r <= arr.len()
// Note: the pivot may get moved
pub fn fat_partition<Element: Clone, Comparator>(
    arr: &mut [Element],
    comparator: &Comparator,
    pivot_index: usize,
) -> (usize, usize)
    where
        Comparator: Fn(&Element, &Element) -> Ordering,
        Element: Clone,
{
    let len = arr.len();
    assert!(len > 2);
    
    // it clones the pivot element to stack
    let pivot = arr[pivot_index].clone();
    
    let mut left_index = 0;
    let mut right_index = len - 1;
    let mut eq_index = 0;
    
    // we want:
    // the "left" region:  arr[0..left_index] < pivot (does not include left_index)
    // the "equal" region: arr[left_index..eq_index] == pivot
    // the "right" region: arr[(right_index+1)..] > pivot (does not include right_index)
    // the region between "equal" region and "right" region is to be processed will shrink to empty.
    while eq_index <= right_index {
        match comparator(&arr[eq_index], &pivot) {
            Ordering::Less => {
                if left_index == eq_index {
                    // if left_index == eq_index, the "equal" region is empty,
                    // given arr[0..left_index] < pivot, arr[left_index] < pivot,
                    // now arr[0..(left_index + 1)] < pivot, we can substitute left_index with left_index+1
                    // the "equal" region should still be empty, so substitute eq_index with eq_index+1
                    left_index += 1;
                    eq_index += 1;
                } else {
                    // the "equal" region is not empty,
                    // given arr[0..left_index] < pivot, arr[left_index..eq_index] == pivot,
                    // arr[eq_index] < pivot, arr[left_index] == pivot,
                    // we can move it to the "left" region, by swapping arr[eq_index] with arr[left_index],
                    // at the same time rotating the "equal" region ("equal" region's first element is now on its tail).
                    arr.swap(eq_index, left_index);
                    // now arr[left_index] < pivot, arr[eq_index] == pivot,
                    // now arr[0..(left_index+1)] < pivot, arr[(left_index+1)..(eq_index+1)] == pivot
                    // we can substitute left_index with left_index+1, eq_index with eq_index+1
                    left_index += 1;
                    eq_index += 1;
                }
            }
            Ordering::Equal => {
                // given arr[left_index..eq_index] == pivot, expand the "equal" region
                // now arr[eq_index] == pivot, we can substitute eq_index with eq_index+1
                eq_index += 1;
            }
            Ordering::Greater => {
                // given arr[(right_index + 1)..] > pivot,
                // given arr[eq_index] > pivot, we move it to the "right" region.
                arr.swap(eq_index, right_index);
                // now arr[right_index] > pivot,
                // now arr[(right_index - 1 + 1)..] > pivot, we can substitute right_index with right_index-1
                right_index -= 1;
            }
        }
    }
    // exiting loop means eq_index > right_index
    
    // in the loop, either eq_index or right_index can only move by one each time, so they cannot be far apart.
    // eq_index == right_index + 1
    assert_eq!(eq_index, right_index + 1);
    
    // now, arr[0..left_index] < pivot, arr[left_index..eq_index] == pivot, arr[eq_index..] > pivot
    (left_index, eq_index)
}

// Fat partition that does not require Clone.
// It returns (l, r) where arr[0..l] < pivot, arr[l..r] == pivot, and arr[r..] > pivot
// Same as the fat partition above, but tracks the pivot's index, instead of copying pivot to stack.
// See the previous function for detailed invariant explanation.
pub fn fat_partition_no_clone_required<Element, Comparator>(
    arr: &mut [Element],
    comparator: &Comparator,
    initial_pivot_index: usize,
) -> (usize, usize)
    where
        Comparator: Fn(&Element, &Element) -> Ordering
{
    let len = arr.len();
    assert!(len > 2);
    
    let mut curr_pivot_index = initial_pivot_index;
    
    let mut left_index = 0;
    let mut right_index = len - 1;
    let mut eq_index = 0;
    
    while eq_index <= right_index {
        if curr_pivot_index == eq_index {
            // no need to compare arr[eq_index] with arr[pivot_index] now, treat it as equal
            eq_index += 1;
            continue;
        }
        // now curr_pivot_index != eq_index
        
        match comparator(&arr[eq_index], &arr[curr_pivot_index]) {
            Ordering::Less => {
                if left_index == eq_index {
                    left_index += 1;
                    eq_index += 1;
                } else {
                    arr.swap(eq_index, left_index);
                    
                    if left_index == curr_pivot_index {
                        // update the pivot index as it has been moved
                        curr_pivot_index = eq_index;
                    }
                    
                    left_index += 1;
                    eq_index += 1;
                }
            }
            Ordering::Equal => {
                eq_index += 1;
            }
            Ordering::Greater => {
                arr.swap(eq_index, right_index);
                
                if right_index == curr_pivot_index {
                    // update the pivot index as it has been moved
                    curr_pivot_index = eq_index;
                }
                
                right_index -= 1;
            }
        }
    }
    
    assert_eq!(eq_index, right_index + 1);
    
    (left_index, eq_index)
}

//noinspection SpellCheckingInspection
//noinspection DuplicatedCode
#[cfg(test)]
mod tests {
    use rand::{Rng, SeedableRng};
    use rand::prelude::StdRng;
    
    use super::*;
    
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
    
    #[test]
    fn test_lomuto_partition() {
        let mut rng = create_rng();
        
        for i in 0..1000 {
            let mut vec = random_vec(&mut rng);
            
            let pivot_index: usize =
                if (0..10).contains(&i) {
                    // chose the minimum element as pivot
                    vec.iter().enumerate().min_by_key(|(_idx, ele)| *ele).unwrap().0
                } else if (10..20).contains(&i) {
                    // chose the maximum element as pivot
                    vec.iter().enumerate().max_by_key(|(_idx, ele)| *ele).unwrap().0
                } else {
                    // choose random element as pivot
                    rng.gen_range(0..vec.len())
                };
            let p = lomuto_partition(vec.as_mut_slice(), &|x: &i32, y: &i32| x.cmp(y), pivot_index);
            assert!(p < vec.len());
            
            let left_max = vec[0..p].iter().max();
            let pivot = vec[p];
            let right_min = vec[(p + 1)..].iter().min();
            
            if let Some(left_max) = left_max {
                assert!(*left_max < pivot);
            }
            
            if let Some(right_min) = right_min {
                assert!(*right_min >= pivot);
            }
        }
    }
    
    #[test]
    fn special_test_hoare_partition() {
        {
            let mut arr = [313, 331, 910, 1368];
            let p0 = _hoare_wikipedia(&mut arr, &|x: &i32, y: &i32| x.cmp(y), 3);
            
            assert!(p0 > 0, "left part is empty");
            assert!(p0 < arr.len(), "right part is empty");
        }
        
        {
            let mut arr = [1, 2, 3];
            let p0 = _hoare_wikipedia(&mut arr, &|x: &i32, y: &i32| x.cmp(y), 0);
            
            assert!(p0 > 0, "left part is empty");
            assert!(p0 < arr.len(), "right part is empty");
        }
    }
    
    #[test]
    fn test_hoare_partition() {
        let mut rng = create_rng();
        
        for i in 0..1000 {
            let mut vec = random_vec(&mut rng);
            
            let pivot_index: usize =
                if (0..10).contains(&i) {
                    // chose the minimum element as pivot
                    vec.iter().enumerate().min_by_key(|(_idx, ele)| *ele).unwrap().0
                } else if (10..20).contains(&i) {
                    // chose the maximum element as pivot
                    vec.iter().enumerate().max_by_key(|(_idx, ele)| *ele).unwrap().0
                } else {
                    // choose random element as pivot
                    rng.gen_range(0..vec.len())
                };
            let p = hoare_partition(vec.as_mut_slice(), &|x: &i32, y: &i32| x.cmp(y), pivot_index);
            // arr[0..p] <= pivot and arr[p..] > pivot
            
            assert_ne!(p, 0, "the left part is empty");
            assert_ne!(p, vec.len(), "the right part is empty");
            
            let left_max = vec[0..p].iter().max().unwrap();
            let right_min = vec[p..].iter().min().unwrap();
            
            assert!(left_max <= right_min);
        }
    }
    
    #[test]
    fn test_fat_partition() {
        let mut rng = create_rng();
        
        for i in 0..1000 {
            let mut vec = random_vec(&mut rng);
            
            let pivot_index: usize =
                if (0..10).contains(&i) {
                    // chose the minimum element as pivot
                    vec.iter().enumerate().min_by_key(|(_idx, ele)| *ele).unwrap().0
                } else if (10..20).contains(&i) {
                    // chose the maximum element as pivot
                    vec.iter().enumerate().max_by_key(|(_idx, ele)| *ele).unwrap().0
                } else {
                    // choose random element as pivot
                    rng.gen_range(0..vec.len())
                };
            
            let (l, r) = fat_partition(vec.as_mut_slice(), &|x: &i32, y: &i32| x.cmp(y), pivot_index);
            
            validate_fat_partition_result(&mut vec, l, r);
        }
        
        for i in 0..1000 {
            let mut vec = random_vec(&mut rng);
            
            let pivot_index: usize =
                if (0..10).contains(&i) {
                    // chose the minimum element as pivot
                    vec.iter().enumerate().min_by_key(|(_idx, ele)| *ele).unwrap().0
                } else if (10..20).contains(&i) {
                    // chose the maximum element as pivot
                    vec.iter().enumerate().max_by_key(|(_idx, ele)| *ele).unwrap().0
                } else {
                    // choose random element as pivot
                    rng.gen_range(0..vec.len())
                };
            
            let (l, r) = fat_partition_no_clone_required(
                vec.as_mut_slice(), &|x: &i32, y: &i32| x.cmp(y), pivot_index,
            );
            
            validate_fat_partition_result(&mut vec, l, r);
        }
    }
    
    fn validate_fat_partition_result(vec: &mut Vec<i32>, l: usize, r: usize) {
        assert!(l < r, "equal region is empty");
        
        let left_max = vec[0..l].iter().max();
        let pivot = vec[l];
        let right_min = vec[r..].iter().min();
        
        assert!(vec[l..r].iter().all(|x| *x == pivot), "equal region is not equal");
        
        if let Some(left_max) = left_max {
            assert!(*left_max < pivot);
        }
        
        if let Some(right_min) = right_min {
            assert!(*right_min >= pivot);
        }
    }
}

fn _hoare_wikipedia<Element: Clone, Comparator>(
    arr: &mut [Element],
    comparator: &Comparator,
    pivot_index: usize,
) -> usize
    where Comparator: Fn(&Element, &Element) -> Ordering,
{
    let len = arr.len();
    let pivot = (&arr[pivot_index]).clone();
    let mut left_index: i32 = -1;
    let mut right_index: i32 = len as i32;
    
    loop {
        left_index += 1;
        while comparator(&arr[left_index as usize], &pivot) == Less {
            left_index += 1;
        }
        right_index -= 1;
        while comparator(&arr[right_index as usize], &pivot) == Greater {
            right_index -= 1;
        }
        if left_index >= right_index {
            if right_index as usize == len - 1 {
                return right_index as usize;
            } else {
                return right_index as usize + 1;
            }
        }
        
        arr.swap(left_index as usize, right_index as usize);
    }
}