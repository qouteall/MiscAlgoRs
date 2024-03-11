use crate::sort::lazy_quick_sort::LazyQuickSorter;


mod sort;


fn main() {
    println!("Hello, world!");

    let mut arr = [1, 3, 2, 4, 5];

    let mut s = LazyQuickSorter::new(&mut arr, &|x: &i32| *x);

    println!("{:?}", s.at(1));

}
