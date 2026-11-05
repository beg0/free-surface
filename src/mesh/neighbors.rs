//! # Neighbors
//!
//! Easy access to neighbor cell (aka neighbor element) in a mesh
//!
use super::celliterator::{CellData, CellIterator};
use crate::storage::selafin::geometry::SlfGeometry;

/// Info about each neighbor of each point in the mesh
pub struct Neighbors {
    /// Neighbors info, indexed by point index
    neighbors: Vec<Option<NeighborInfo>>,

    /// Number of point in a cell (aka element)
    point_per_cell: usize,

    /// Maximum number of cell a point can be in (cached value)
    max_cell_per_point: usize,

    /// Maximun number of points a point can be connected with
    max_neighbor_cnt: usize,
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

    /// Create a [Neighbors] from a selafin [SlfGeometry].
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

        for pt_idx in geometry.ikle2(0).expect("No layer #0") {
            // For a given cell with point A, B, C, edges are the following:
            //  * A-B
            //  * B-C
            //  * C-A
            // Thus each point has 2 neighbors.
            neighbors_count[(*pt_idx) as usize] += 2;
        }

        // As per above formula, a point is counted twice each time it is part of a cell
        // thus to know in how many cell a point is, we have to divide neighbors_count[pt_idx] by two
        //  Below, instead of dividing every value by two, we simply divide the result by 2

        let boundary_info = geometry.ipob2(0).expect("No layer #0");

        let max_cell_per_point = neighbors_count
            .iter()
            .fold(0, |current, candidate_value| current.max(*candidate_value))
            / 2;

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

        let neighbor_cnt_per_point = neighbor_info_per_point.iter().map(|ni| ni.len());
        // let max_neighbor_cnt =
        //     neighbor_cnt_per_point.fold(0, |current, candidate| current.max(candidate));

        // TODO: telemac looks for a unique neighbor only in tetrahedra type of element (e.g. IELM==31)
        // otherwise, it takes max_cell_per_point as a basis, and adding 1 for boundary node.
        // In case of tetrahedra, telemac does not look at boundary info (which sound strange to me)
        let max_neighbor_cnt = neighbor_cnt_per_point.enumerate().fold(
            0,
            |current, (candidate_idx, candidate_value)| {
                let mut v = candidate_value;
                // If this node is at boundary, we need to consider a (fantom) neighbor point from outside the mesh
                if boundary_info[candidate_idx] != 0 {
                    v += 1;
                }

                current.max(v)
            },
        );

        Self {
            neighbors,
            point_per_cell,
            max_cell_per_point,
            max_neighbor_cnt,
        }
    }

    /// Maximum number of cell a point can be in
    ///
    /// This value can be used as an upper bound for allocations and/or loop
    /// iteration.
    ///
    /// # Formula
    ///
    /// This value can be mathematically defined as following
    ///
    /// $$\underset{pt \in P}{max} ( card \lbrace c \in M, pt \in c \rbrace )$$
    ///
    /// Where:
    /// - $P$ is the set of points in the mesh
    /// - $M$ is the mesh a.k.a the set of cells
    ///
    /// # Performance
    ///
    /// This is a cached value.
    pub fn max_cell_per_point(&self) -> usize {
        self.max_cell_per_point
    }

    /// Maximun number of points a point can be connected with
    ///
    /// This is also the maximun number of (unique) edges a point can be part of
    ///
    /// By unique edge we mean: if edge [i-j] is the edge of 2 cells, it is count only once
    ///
    /// # Formula
    ///
    /// This value can be mathematically defined as following
    ///
    /// $$\underset{pt \in P}{max} ( card \lbrace e \in E, pt \in e \rbrace )$$
    ///
    /// Where:
    /// - $P$ is the set of points in the mesh
    /// - $E$ is the set of unique edges in the mesh
    ///
    /// # Performance
    ///
    /// This is a cached value.
    pub fn max_neighbor_cnt(&self) -> usize {
        self.max_neighbor_cnt
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
