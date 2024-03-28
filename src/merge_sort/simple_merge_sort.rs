use std::cmp::Ordering;

use crate::merge_sort::merge::{merge_two_sorted_sequences, smart_merge_two_adjacent_sorted_sequences_inplace};

// It does not modify the input array, it creates a new Vec.
// This requires Clone.
fn simple_merge_sort_requires_clone<Element: Clone, Comparator>(
    arr: &[Element], compare: &Comparator,
) -> Vec<Element>
    where
        Comparator: Fn(&Element, &Element) -> Ordering
{
    if arr.len() <= 1 {
        return arr.to_vec();
    }
    
    let mid = arr.len() / 2;
    
    let left = simple_merge_sort_requires_clone(&arr[..mid], compare);
    let right = simple_merge_sort_requires_clone(&arr[mid..], compare);
    
    let mut result = Vec::with_capacity(arr.len());
    
    merge_two_sorted_sequences(&left, &right, compare, &mut |_, element| {
        result.push(element.clone());
    });
    
    result
}

// this modifies the slice in-place.
// this does not require Clone.
pub fn simple_merge_sort_inplace<Element, Comparator>(
    arr: &mut [Element], compare: &Comparator,
)
    where
        Comparator: Fn(&Element, &Element) -> Ordering
{
    if arr.len() <= 1 {
        return;
    }
    
    let mid = arr.len() / 2;
    
    simple_merge_sort_inplace(&mut arr[..mid], compare);
    
    simple_merge_sort_inplace(&mut arr[mid..], compare);
    
    smart_merge_two_adjacent_sorted_sequences_inplace(
        arr, mid, compare,
    );
}

#[cfg(test)]
mod tests {
    use rand::{Rng, SeedableRng};
    use rand::prelude::StdRng;
    
    use super::*;
    
    fn random_int_vec(rng: &mut StdRng) -> Vec<i32> {
        let len = rng.gen_range(0..1000);
        let max = rng.gen_range(1..1000);
        
        return (0..len).map(|_| rng.gen_range(0..max)).collect();
    }
    
    fn random_string_vec(rng: &mut StdRng) -> Vec<String> {
        let len = rng.gen_range(0..1000);
        let max = rng.gen_range(1..1000);
        
        return (0..len).map(|_| {
            let len = rng.gen_range(1..10);
            (0..len).map(|_| rng.gen_range(('a' as u8)..=('z' as u8)) as char).collect()
        }).collect();
    }
    
    #[test]
    fn test_simple_merge_sort_requires_clone() {
        let mut rng = SeedableRng::seed_from_u64(123456);
        
        for _i in 0..1000 {
            let mut vec = random_int_vec(&mut rng);
            
            let sorted = simple_merge_sort_requires_clone(vec.as_slice(), &|a, b| a.cmp(b));
            
            vec.sort();
            
            assert_eq!(sorted, vec);
        }
        
        for _i in 0..1000 {
            let mut vec = random_string_vec(&mut rng);
            
            let sorted = simple_merge_sort_requires_clone(
                vec.as_slice(), &|a, b| a.len().cmp(&b.len()),
            );
            
            vec.sort_by(&|a: &String, b: &String| a.len().cmp(&b.len()));
            
            assert_eq!(sorted, vec);
        }
    }
    
    // #[test]
    // fn test_stability() {
    //     let mut v: Vec<String> = vec!["add", "what", "o", "c", "nn", "d", "ff"].iter()
    //         .map(|s|s.to_string()).collect();
    //     let mut v_ref = v.clone();
    //     
    //     let sorted = simple_merge_sort_inplace(
    //         v.as_mut_slice(), &|a, b| a.len().cmp(&b.len())
    //     );
    //     
    //     v_ref.sort_by(&|a: &String, b: &String| a.len().cmp(&b.len()));
    //     
    //     assert_eq!(v, v_ref);
    // }
    #[test]
    fn test_simple_merge_sort_inplace() {
        let mut rng = SeedableRng::seed_from_u64(123456);
        
        for _i in 0..1000 {
            let mut vec = random_int_vec(&mut rng);
            let mut vec_ref = vec.clone();
            
            simple_merge_sort_inplace(vec.as_mut_slice(), &|a, b| a.cmp(b));
            
            vec_ref.sort();
            
            assert_eq!(vec, vec_ref);
        }
        
        for _i in 0..1000 {
            let mut vec = random_string_vec(&mut rng);
            let mut vec_ref = vec.clone();
            
            simple_merge_sort_inplace(
                vec.as_mut_slice(), &|a, b| a.len().cmp(&b.len()),
            );
            
            vec_ref.sort_by(&|a: &String, b: &String| a.len().cmp(&b.len()));
            
            assert_eq!(vec, vec_ref);
        }
    }
}
