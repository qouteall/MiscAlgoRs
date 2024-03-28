use std::cmp::Ordering;

use crate::quick_sort::partition::fat_partition_no_clone_required;
use crate::quick_sort::pivot_select::median_of_three_pivot;

pub fn normal_quick_sort<Element, Comparator>(
    arr: &mut [Element], compare: &Comparator,
) where
    Comparator: Fn(&Element, &Element) -> Ordering,
{
    let len = arr.len();
    
    if len <= 1 {
        return;
    }
    
    if len == 2 {
        if compare(&arr[0], &arr[1]) == Ordering::Greater {
            arr.swap(0, 1);
        }
        return;
    }
    
    let initial_pivot_index = median_of_three_pivot(arr, compare);
    
    let (l, r) = fat_partition_no_clone_required(arr, compare, initial_pivot_index);
    
    let left_part = &mut arr[0..l];
    normal_quick_sort(left_part, compare);
    
    let right_part = &mut arr[r..];
    normal_quick_sort(right_part, compare);
}


#[cfg(test)]
mod tests {
    use rand::{Rng, rngs::StdRng, SeedableRng};
    
    use super::*;
    
    #[test]
    fn test_normal_quick_sort() {
        let mut rng = create_rng();
        
        for _i in 0..1000 {
            let mut vec = random_vec(&mut rng);
            let mut vec_ref = vec.clone();
            
            let slice = vec.as_mut_slice();
            
            normal_quick_sort(slice, &|a, b| a.cmp(b));
            
            vec_ref.sort();
            
            assert_eq!(vec, vec_ref);
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
}