




// simple quick sort
fn simple_quick_sort(arr: &mut [i32]) {
    if arr.len() <= 1 {
        return;
    }
    let pivot = arr[arr.len() / 2];
    let mut left_index = 0;
    let mut right_index = arr.len() - 1;
    while left_index < right_index {
        while arr[left_index] < pivot {
            left_index += 1;
        }
        while arr[right_index] > pivot {
            right_index -= 1;
        }
        if left_index <= right_index {
            arr.swap(left_index, right_index);
            left_index += 1;
            right_index -= 1;
        }
    }

    if right_index > 0 {
        simple_quick_sort(&mut arr[0..=right_index]);
    }

    if left_index < arr.len() {
        simple_quick_sort(&mut arr[left_index..]);
    }
}

// generic quick sort
fn generic_quick_sort<Element, SortKey>(
    arr: &mut [Element],
    extract_sort_key: impl Fn(&Element) -> SortKey,
) where
    SortKey: Ord,
{
    if arr.len() <= 1 {
        return;
    }

    let pivot_index = arr.len() / 2;
    let mut left_index = 0;
    let mut right_index = arr.len() - 1;
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

    if right_index > 0 {
        generic_quick_sort(&mut arr[0..=right_index], &extract_sort_key);
    }

    if left_index < arr.len() {
        generic_quick_sort(&mut arr[left_index..], &extract_sort_key);
    }
}
