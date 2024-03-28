#![allow(clippy::needless_lifetimes)]

use std::cmp::Ordering;
use std::collections::HashMap;
use std::marker::PhantomData;

use crate::data_structure::dag::DAGTraverser;
use crate::functional::lazy_eval::FuncHavingFixedPointMut;

pub trait DistanceOps<EdgeData, Distance> {
    fn get_distance(&self, edge: &EdgeData) -> Distance;
    
    fn add_distance(&self,a: &Distance, b: &Distance) -> Distance;
    
    fn zero_distance(&self) -> Distance;
    
    fn compare_distance(&self,a: &Distance, b: &Distance) -> Ordering;
}

pub struct DagShortestPathSolver<
    NodeRef, EdgeData, Distance,
    Traverser: DAGTraverser<NodeRef, EdgeData>,
    DistanceOpsImpl: DistanceOps<EdgeData, Distance>
>
{
    traverser: Traverser,
    distance_ops: DistanceOpsImpl,
    _phantom: PhantomData<(NodeRef, EdgeData, Distance)>,
}

impl<
    NodeRef, EdgeData, Distance,
    Traverser: DAGTraverser<NodeRef, EdgeData>,
    DistanceOpsImpl: DistanceOps<EdgeData, Distance>
> DagShortestPathSolver<NodeRef, EdgeData, Distance, Traverser, DistanceOpsImpl> {
    pub fn new(traverser: Traverser, distance_ops: DistanceOpsImpl) -> Self {
        Self {
            traverser,
            distance_ops,
            _phantom: PhantomData,
        }
    }
}

#[derive(Clone)]
#[derive(Debug)]
#[derive(PartialEq)]
pub struct PathInfo<NodeRef: Clone, Distance: Clone> {
    next_node: NodeRef,
    distance_to_destination: Distance,
}

impl<
    NodeRef: Clone+Eq, EdgeData, Distance:Clone,
    Traverser: DAGTraverser<NodeRef, EdgeData>,
    DistanceOpsImpl: DistanceOps<EdgeData, Distance>
> FuncHavingFixedPointMut<(NodeRef, NodeRef), Option<PathInfo<NodeRef, Distance>>>
for DagShortestPathSolver<NodeRef, EdgeData, Distance, Traverser, DistanceOpsImpl>
{
    fn eval<FuncArg>(
        &self, recursion: &mut FuncArg,
        two_ends: &(NodeRef, NodeRef),
    ) -> Option<PathInfo<NodeRef, Distance>>
        where FuncArg: FnMut(&(NodeRef, NodeRef)) -> Option<PathInfo<NodeRef, Distance>>
    {
        let (src, dst) = two_ends;
        if src == dst {
            return Some(PathInfo {
                next_node: dst.clone(),
                distance_to_destination: self.distance_ops.zero_distance(),
            });
        }
        self.traverser.get_edges_coming_out(src.clone())
            .filter_map(|(edge_data, next_node)| -> Option<PathInfo<NodeRef, Distance>> {
                let next_node_to_dest_info: Option<PathInfo<NodeRef, Distance>> =
                    recursion(&(next_node.clone(), dst.clone()));
                match next_node_to_dest_info {
                    Some(next_node_path_info) => {
                        let next_node_distance_to_dest = next_node_path_info.distance_to_destination.clone();
                        let edge_distance = self.distance_ops.get_distance(&edge_data);
                        let new_distance =
                            self.distance_ops.add_distance(&edge_distance, &next_node_distance_to_dest);
                        Some(PathInfo { next_node, distance_to_destination: new_distance })
                    }
                    None => None,
                }
            })
            .min_by(|a, b| {
                self.distance_ops.compare_distance(&a.distance_to_destination, &b.distance_to_destination)
            })
    }
}

struct I32DistanceOps {}

impl DistanceOps<i32, i32> for I32DistanceOps {
    fn get_distance(&self, edge: &i32) -> i32 {
        // in the test case the edge is just the distance
        // in complex case it will extract the distance from the edge data
        *edge
    }
    
    fn add_distance(&self, a: &i32, b: &i32) -> i32 {
        a + b
    }
    
    fn zero_distance(&self) -> i32 {
        0
    }
    
    fn compare_distance(&self, a: &i32, b: &i32) -> Ordering {
        a.cmp(b)
    }
}

struct F64DistanceOps {}

impl DistanceOps<f64, f64> for F64DistanceOps {
    fn get_distance(&self, edge: &f64) -> f64 {
        *edge
    }
    
    fn add_distance(&self, a: &f64, b: &f64) -> f64 {
        a + b
    }
    
    fn zero_distance(&self) -> f64 {
        0.0
    }
    
    fn compare_distance(&self, a: &f64, b: &f64) -> Ordering {
        a.partial_cmp(b).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use crate::data_structure::dag::HashMapDAG;
    use crate::data_structure::matrix2d::Matrix2D;
    use crate::functional::lazy_eval::LazyEvalFixedPointApplyFunc;
    
    use super::*;
    
    fn init_graph(edges: Vec<(&'static str, &'static str, i32)>) -> HashMapDAG<&'static str, i32> {
        let mut graph: HashMapDAG<&str, i32> = HashMap::new();
        for (src, dst, edge_data) in edges {
            graph.entry(src).or_insert(HashMap::new()).insert(dst, edge_data);
        }
        graph
    }
    
    #[test]
    fn test_dag_shortest_path_1() {
        let graph = init_graph(vec![
            ("a", "b", 1),
            ("a", "c", 2),
            ("b", "c", 3),
            ("b", "d", 4),
            ("c", "d", 5),
        ]);
        let solver = DagShortestPathSolver::new(graph, I32DistanceOps {});
        let mut cache: HashMap<(&str, &str), Option<PathInfo<&str, i32>>> = HashMap::new();
        let mut cached_solver = LazyEvalFixedPointApplyFunc::new(&solver, cache);
        let result = cached_solver(&("a", "d"));
        assert_eq!(result, Some(PathInfo { next_node: "b", distance_to_destination: 5 }));
    }
    
    #[test]
    fn test_dag_shortest_path_2() {
        let mut matrix: Matrix2D<Option<f64>> = Matrix2D::new_defaulted(4, 4);
        matrix.set(0, 1, Some(1.0));
        matrix.set(0, 2, Some(2.0));
        matrix.set(1, 2, Some(3.0));
        matrix.set(1, 3, Some(4.0));
        matrix.set(2, 3, Some(5.0));
        let solver = DagShortestPathSolver::new(matrix, F64DistanceOps {});
        // it needs two layers of Option, because cache slot is optional and reachability is also optional
        let cache: Matrix2D<Option<Option<PathInfo<usize, f64>>>> = Matrix2D::new_defaulted(4, 4);
        let mut cached_solver = LazyEvalFixedPointApplyFunc::new(&solver, cache);
        let result = cached_solver(&(0, 3));
        assert_eq!(result, Some(PathInfo { next_node: 1, distance_to_destination: 5.0 }));
    }
}
