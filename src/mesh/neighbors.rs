//! # Neighbors
//!
//! Easy access to neighbor cell in a mesh
//!
use super::celliterator::{CellData, CellIterator};
use crate::storage::selafin::geometry::SlfGeometry;
pub struct Neighbors {
    neighbors: Vec<Option<NeighborInfo>>,
    point_per_cell: usize,
}

/// For each edge in a cell, tell who is my neighbor cell
#[derive(Clone, Debug)]
pub struct NeighborInfo {
    /// Index of the neighbors cell
    pub cell_idx: usize,

    /// Edge of the neighbor cell that share the border with us
    pub edge_idx: usize,
}

#[derive(Debug)]
struct NeighborInfoWithEdge {
    /// The index of the other point in the edge. The first point is given by the index in the containing vector
    pair_edge_idx: usize,

    /// Who is my neighnor
    info: NeighborInfo,
}

pub type NeighborsIterator<'a> = CellIterator<'a, Neighbors>;

impl Neighbors {
    pub fn iter<'a>(&'a self) -> NeighborsIterator<'a> {
        NeighborsIterator::new(self)
    }

    // fn linear_idx(&self, cell_idx: usize, edge_idx: usize) -> usize {
    //     (cell_idx * self.point_per_cell) + edge_idx
    // }

    pub fn from_selafin(geometry: &SlfGeometry) -> Self {
        // Make sure we are in 2D
        assert_eq!(geometry.planes_cnt(), 1);

        // In 2D, the number of edge per cell is the same as the number of point per cell
        // but this assumption in wrong in 3D
        let edge_per_cell = geometry.point_per_element();
        let point_per_cell = geometry.point_per_element();
        let points_count = geometry.points_count();

        let cell_cnt = geometry.elements_per_layer();

        let linear_idx = |cell_idx: usize, edge_idx: usize| (cell_idx * point_per_cell) + edge_idx;

        // Pre-allocate all the neighbors
        let mut neighbors: Vec<Option<NeighborInfo>> =
            vec![None; geometry.elements_count() * edge_per_cell];

        // Compute how many neighbors each points have

        let mut neighbors_count: Vec<usize> = vec![0; points_count];

        // for points_in_cell in geometry.ikle2(0).expect("No layer #0").chunks(point_per_cell) {
        //     for idx_in_cell in 0..(edge_per_cell-1) {
        //         let start_point = points_in_cell[idx_in_cell] as usize;
        //         let end_point = points_in_cell[(idx_in_cell + 1) % point_per_cell] as usize;

        //         // TODO: improve me. end_point for one edge will be the start_point for next edge
        //         neighbors_count[start_point] += 1;
        //         neighbors_count[end_point] += 1;
        //     }
        // }

        for pt_idx in geometry.ikle2(0).expect("No layer #0") {
            // For a given cell with point A, B, C, edges are the following:
            //  * A-B
            //  * B-C
            //  * C-A
            // Thus each point has 2 neighbors.
            neighbors_count[(*pt_idx) as usize] += 2;
        }

        // let mut start_idx_offset: Vec<usize> = Vec::with_capacity(points_count);
        // let mut sum = 0;
        // start_idx_offset.push(sum);
        // for i in 0..points_count-1 {
        //     sum += neighbors_count[i];
        //     start_idx_offset.push(sum);
        // }
        // let imax = sum + neighbors_count.last().unwrap();

        // TODO: check what is the best (fastest) storage for neighbor_info_per_point:
        //   - Vec<Vec<_>>, knowing that neighbors_count[i] is probably oversized
        //   - Vec<_> with size = sum(neighbors_count[i]). A single (too big) allocation but more math to reach each
        //     neighborInfo
        //   - Vec<LinkedList> No extra memory allocation needed. No need to compute `neighbors_count` but lots
        //     of allocation
        let mut neighbor_info_per_point: Vec<Vec<NeighborInfoWithEdge>> =
            Vec::with_capacity(points_count);
        for cnt in neighbors_count {
            neighbor_info_per_point.push(Vec::with_capacity(cnt));
        }

        let ikle = geometry.ikle2(0).expect("No layer #0");
        for cell_idx in 0..cell_cnt {
            for edge_idx_in_cell in 0..edge_per_cell {
                let start_point = ikle[linear_idx(cell_idx, edge_idx_in_cell)] as usize;
                let end_point =
                    ikle[linear_idx(cell_idx, (edge_idx_in_cell + 1) % point_per_cell)] as usize;

                // In order to easily compare edges, we need to make sure
                // the two end are always ordered in the same order
                // indeed, consider 2 cells:
                // - first with points A, B, C (in that order)
                // - second with points D, B, A (in that order)
                //
                // Both cells will be neighbors, but to easily compare the edge definition we fix an order
                // For example (A, B) (if A < B)
                //
                let (lower_idx, upper_idx) = if start_point < end_point {
                    (start_point, end_point)
                } else {
                    (end_point, start_point)
                };

                let neighbor_info_lower_point = &mut neighbor_info_per_point[lower_idx];

                match neighbor_info_lower_point
                    .iter()
                    .find(|info| info.pair_edge_idx == upper_idx)
                {
                    Some(existing_neighbor) => {
                        // Save who is my neighbor
                        neighbors[linear_idx(cell_idx, edge_idx_in_cell)] =
                            Some(existing_neighbor.info.clone());

                        // I'm the neighbor of my neighbor
                        neighbors[linear_idx(
                            existing_neighbor.info.cell_idx,
                            existing_neighbor.info.edge_idx,
                        )] = Some(NeighborInfo {
                            cell_idx,
                            edge_idx: edge_idx_in_cell,
                        });
                    }
                    None => neighbor_info_lower_point.push(NeighborInfoWithEdge {
                        pair_edge_idx: upper_idx,
                        info: NeighborInfo {
                            edge_idx: edge_idx_in_cell,
                            cell_idx,
                        },
                    }),
                }
            }
        }

        //dbg!(neighbor_info_per_point);

        Self {
            neighbors,
            point_per_cell,
        }
    }
}

impl CellData for Neighbors {
    type Item = Option<NeighborInfo>;

    fn point_per_cell(&self) -> usize {
        self.point_per_cell
    }

    fn data_len(&self) -> usize {
        self.neighbors.len()
    }

    fn point_data(&self, range: std::ops::Range<usize>) -> &[Option<NeighborInfo>] {
        &self.neighbors[range]
    }
}
