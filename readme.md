
Some algorithms written in Rust, in generic ways.

It contains:

- Quick sort
  - Lomuto partition, Hoare partition and Fat partition
  - Normal quick sort
  - Lazy quick sort (quick select)
  - Functional-style stable quick sort
  - Container-agnostic quick sort that works on both linked list and array
- Merge sort
  - Normal merge sort
  - Simple concurrent merge sort
- Functional programming things
  - Lazy evaluation
  - Y combinator
- Data structure
  - Linked list (implemented using slotmap, a kind of arena)
  - Min heap
  - Abstracted DAG and DAG traversal trait (generic to graph implementation)
- Dynamic programming
  - Shortest path in DAG (generic to graph implementation)
  - TODO
