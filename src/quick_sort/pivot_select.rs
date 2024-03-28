use std::cmp::Ordering;

// select the first element as pivot
pub fn first_element_as_pivot<Element>(_arr: &[Element]) -> usize {
    0
}

// select the middle element as pivot
pub fn middle_element_as_pivot<Element>(arr: &[Element]) -> usize {
    arr.len() / 2
}

// select the last element as pivot
pub fn last_element_as_pivot<Element>(arr: &[Element]) -> usize {
    arr.len() - 1
}

// select the median of the first, middle, and last element as pivot
pub fn median_of_three_pivot<Element, Comparator>(
    arr: &[Element],
    compare: &Comparator,
) -> usize
    where
        Comparator: Fn(&Element, &Element) -> Ordering,
{
    let len = arr.len();
    let i1 = 0;
    let i2 = len / 2;
    let i3 = len - 1;
    let e1 = &arr[i1];
    let e2 = &arr[i2];
    let e3 = &arr[i3];
    
    let cmp12 = compare(e1, e2);
    let cmp23 = compare(e2, e3);
    
    // e1 <= e2 <= e3
    if cmp12.is_le() && cmp23.is_le() {
        return i2;
    }
    // e3 <= e2 <= e1
    if cmp12.is_ge() && cmp23.is_ge() {
        return i2;
    }
    
    // only do the third comparison if necessary
    let cmp13 = compare(e1, e3);
    
    // e2 <= e1 <= e3
    if cmp12.is_ge() && cmp13.is_le() {
        return i1;
    }
    // e3 <= e1 <= e2
    if cmp13.is_ge() && cmp23.is_le() {
        return i1;
    }
    
    return i3;
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_median_of_three_pivot() {
        test_median_for(&[3, 2, 1, 4, 5], 0);
        test_median_for(&[1, 2, 5, 4, 3], 4);
        test_median_for(&[1, 2, 3, 4, 5], 2);
        test_median_for(&[5, 4, 3, 2, 1], 2);
    }
    
    fn test_median_for(arr: &[i32], result: usize) {
        let compare = |a: &i32, b: &i32| a.cmp(b);
        assert_eq!(median_of_three_pivot(&arr, &compare), result);
    }
}