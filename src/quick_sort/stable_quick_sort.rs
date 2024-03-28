use std::cmp::Ordering;
use std::cmp::Ordering::Less;
use std::iter::once;

// Stable quick sort in functional programming style.
// Instead of mutating the input array, it creates new arrays.
// It's stable.
// It's slower than the in-place quick sort, because it involves more allocation, copying and comparisons.
fn functional_style_stable_quick_sort<Element: Clone, Comparator>(
    arr: &[Element], compare: &Comparator,
) -> Vec<Element>
    where
        Comparator: Fn(&Element, &Element) -> Ordering
{
    if arr.is_empty() {
        return Vec::new();
    }
    
    // choose first element as pivot, for simplicity
    let pivot = &arr[0];
    
    // the rest of the elements
    let rest = &arr[1..];
    
    // the elements that < pivot (order of equal elements is preserved)
    let left: Vec<_> = rest.iter()
        .filter(|x| compare(x, pivot) == Less)
        .cloned().collect();
    
    // the elements that >= pivot (order of equal elements is preserved)
    let right: Vec<_> = rest.iter()
        .filter(|x| compare(x, pivot) != Less)
        .cloned().collect();
    
    let left_sorted = functional_style_stable_quick_sort(&left, compare);
    
    let right_sorted = functional_style_stable_quick_sort(&right, compare);
    
    // concat [left_sorted..., pivot, right_sorted...]
    left_sorted.into_iter()
        .chain(once(pivot.clone()))
        .chain(right_sorted).collect()
    
    // we want the sort to be stable.
    // the pivot was selected from the first element, so it's the leftest element that equals the pivot,
    // so the elements that are equal to pivot should be on the right part, the right side of the pivot.
    // if pivot was selected from the last element, the equal elements should be on the left part
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_functional_style_stable_quick_sort() {
        let mut vec = vec![
            "apple", "banana", ".", "124", "12345", "orange", "_",
        ];
        
        // sort by string length
        let sorted = functional_style_stable_quick_sort(
            vec.as_slice(), &|a, b| a.len().cmp(&b.len()),
        );
        
        vec.sort_by(&|a: &&str, b: &&str| a.len().cmp(&b.len()));
        
        assert_eq!(sorted, vec);
    }
}