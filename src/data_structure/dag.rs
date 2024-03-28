use std::collections;
use std::collections::HashMap;
use std::hash::Hash;

use crate::data_structure::matrix2d::Matrix2D;

// Provide the interface of traversing a directed acyclic graph.
pub trait DAGTraverser<NodeRef, EdgeData> {
    type EdgeIter<'a>: Iterator<Item=(EdgeData, NodeRef)> where Self: 'a;
    
    fn get_edges_coming_out<'a>(&'a self, n: NodeRef) -> Self::EdgeIter<'a>;
}

// map[src][dst] = edge_data
pub type HashMapDAG<NodeRef, EdgeData> = HashMap<NodeRef, HashMap<NodeRef, EdgeData>>;

impl<NodeRef: Eq + Hash + Clone, EdgeData: Clone> DAGTraverser<NodeRef, EdgeData> for HashMapDAG<NodeRef, EdgeData> {
    type EdgeIter<'a> = std::iter::Map<
        collections::hash_map::Iter<'a, NodeRef, EdgeData>,
        fn((&NodeRef, &EdgeData)) -> (EdgeData, NodeRef)
    > where Self: 'a;
    
    fn get_edges_coming_out<'a>(&'a self, n: NodeRef) -> Self::EdgeIter<'a> {
        let iter = self.get(&n).unwrap().iter();
        iter.map(|(dst, edge_data): (&NodeRef, &EdgeData)| -> (EdgeData, NodeRef) {
            (edge_data.clone(), dst.clone())
        })
    }
}

// matrix[row][col] = edge_data, where row index is src node and column index is dst node.
pub type MatrixDAG<EdgeData> = Matrix2D<Option<EdgeData>>;

impl<EdgeData: Clone> DAGTraverser<usize, EdgeData> for MatrixDAG<EdgeData> {
    type EdgeIter<'a> = std::iter::FilterMap<
        std::iter::Enumerate<std::slice::Iter<'a, Option<EdgeData>>>,
        fn((usize, &Option<EdgeData>)) -> Option<(EdgeData, usize)>
    > where Self: 'a;
    
    fn get_edges_coming_out<'a>(&'a self, n: usize) -> Self::EdgeIter<'a> {
        self.borrow_row(n).iter().enumerate().filter_map(
            |(dst, edge_data_option): (usize, &Option<EdgeData>)| {
                if let Some(value) = edge_data_option {
                    Some((value.clone(), dst))
                } else {
                    None
                }
            }
        )
    }
}