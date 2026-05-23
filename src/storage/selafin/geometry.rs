//! # Geometry elements in Selfain files

use super::container::SlfArray2D;

/// Coords and mesh for digital elevation model in Selafin
#[derive(Debug)]
pub struct SlfGeometry {
    /// Number of points per elements
    /// Typically 3 in 2D and 6 in 3D
    npd3: usize,

    /// Number of planes (3D computation)
    nplan: u32,

    /// The connectivity table, size of 'nelem3' * 'npd3'
    /// Indexes of nodes to connect each nodes together (0-based indexes)
    mesh: Vec<u32>,

    /// Indexes of nodes at the boundary (0-based indexes)
    /// The value of an element is 0 for an inner point and yields the edge
    /// point numbers for the others),
    ipob3: Vec<u32>,

    /// Coordinates of each points of the mesh
    points: SlfArray2D,

    /// Number of boundaries (for parallel computation)
    pub boundaries_count: u32,

    /// Number of interfaces (for parallel computation)
    pub interfaces_count: u32,
}

impl Default for SlfGeometry {
    fn default() -> Self {
        SlfGeometry {
            npd3: 3,
            nplan: 1,
            mesh: Vec::new(),
            ipob3: Vec::new(),
            points: SlfArray2D::default(),
            boundaries_count: 0,
            interfaces_count: 0,
        }
    }
}

impl SlfGeometry {
    /// Constructor
    pub fn new(
        points: SlfArray2D,
        ipob3: Vec<u32>,
        mesh: Vec<u32>,
        npd3: usize,
        nplan: u32,
    ) -> Self {
        debug_assert!(npd3 > 0);
        debug_assert!(nplan > 0);
        debug_assert!(points.len() >= ipob3.len()); // More points than boundaries
        debug_assert!(mesh.len().is_multiple_of(npd3));
        SlfGeometry {
            npd3,
            nplan,
            mesh,
            ipob3,
            points,
            boundaries_count: 0,
            interfaces_count: 0,
        }
    }

    #[allow(dead_code)]
    pub fn with_parallel_info(mut self, boundaries: u32, interfaces: u32) -> Self {
        self.boundaries_count = boundaries;
        self.interfaces_count = interfaces;
        self
    }

    /// Tell if the Selafin is for 2D of 3D computation
    pub fn dimension(&self) -> u32 {
        if self.nplan > 1 {
            3
        } else {
            2
        }
    }

    /// Number of plane (layer) in  the Selafin file
    /// Always 1 for 2D files, always >= 2 for 3D files
    pub fn planes_cnt(&self) -> u32 {
        self.nplan
    }

    /// Number of points in a 2D plane (layer)
    ///
    /// For 2D selafin, this is the same as [self.points_count]
    /// For 3D Selafin, this is the total number of points ([self.points_count]) divided by the number of
    /// layer as there is the same number of points for each layer.
    pub fn points_per_layer(&self) -> usize {
        if self.nplan > 1 {
            // The number of points is the same for every layer (regular mesh)
            self.points.len() / (self.nplan as usize)
        } else {
            self.points.len()
        }
    }

    /// Alias for [self.points_per_layer] for Telemac compatibility
    pub fn npoin2(&self) -> usize {
        self.points_per_layer()
    }

    /// Total number of points in the mesh
    pub fn points_count(&self) -> usize {
        self.points.len()
    }

    /// Alias for [self.points_count] for Telemac compatibility
    pub fn npoin3(&self) -> usize {
        self.points.len()
    }

    /// Number of triangular in a 2D plane (layer)
    ///
    /// For 2D selafin, this is the same as [self.elements_count]
    /// For 3D Selafin, this is the total number of elements ([self.elements_count]) divided by the number of
    /// layer as there is the same number of element for each layer.
    pub fn elements_per_layer(&self) -> usize {
        let nelem3 = self.elements_count();
        if self.nplan > 1 {
            // The number of points is the same for every layer (regular mesh)
            nelem3 / (self.nplan as usize - 1)
        } else {
            nelem3
        }
    }

    /// Alias for [self.elements_per_layer] for Telemac compatibility
    pub fn nelem2(&self) -> usize {
        self.elements_per_layer()
    }

    /// Total number of element (triangular or prism) or in the mesh
    pub fn elements_count(&self) -> usize {
        self.mesh.len() / self.npd3
    }

    /// Number of points per element on a layer
    ///
    /// In 2D, this is the same as the number of points per layer
    /// In 3D, it's half the number of point per element as there is no need
    /// to connect to upper layer
    pub fn point_per_layer_element(&self) -> usize {
        if self.nplan > 1 {
            self.npd3 / 2 // triangular prism: 6 nodes → 3 in 2-D
        } else {
            self.npd3
        }
    }

    /// Alias for [self.point_per_layer_element] for Telemac compatibility
    pub fn npd2(&self) -> usize {
        self.point_per_layer_element()
    }

    pub fn point_per_element(&self) -> usize {
        self.npd3
    }

    /// Return elements of a single layer
    ///
    /// Layer 0 is the bottom layer, layer `self.planes_cnt() - 1` is the upper layer
    /// For 2D selafin, only layer 0 is valid
    pub fn ikle2(&self, layer: usize) -> Option<&[u32]> {
        let points_per_layer = self.elements_per_layer() * self.point_per_layer_element();
        if layer >= self.nplan as usize {
            None
        } else {
            Some(&self.mesh[layer * points_per_layer..(layer + 1) * points_per_layer])
        }
    }

    /// Return all elements of the selafin
    pub fn ikle3<const N: usize>(&self) -> Option<&[u32; N]> {
        self.mesh.as_array()
    }

    /// Return elements of a single layer
    ///
    /// Layer 0 is the bottom layer, layer `self.planes_cnt() - 1` is the upper layer
    /// For 2D selafin, only layer 0 is valid
    pub fn ipob2(&self, layer: usize) -> Option<&[u32]> {
        let points_per_layer = self.elements_per_layer() * self.point_per_layer_element();
        if layer >= self.nplan as usize {
            None
        } else {
            Some(&self.mesh[layer * points_per_layer..(layer + 1) * points_per_layer])
        }
    }

    /// Return all elements of the selafin
    pub fn ipob3<const N: usize>(&self) -> Option<&[u32; N]> {
        self.ipob3.as_array()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // Test helpers
    // -----------------------------------------------------------------------

    /// Build a minimal 2D geometry (nplan = 1, triangles: npd3 = 3).
    ///
    ///  Points: 4 points arranged in a square
    ///  Elements: 2 triangles
    ///
    ///   3 --- 2
    ///   |  \  |
    ///   0 --- 1
    ///
    ///  Triangle 0: [0, 1, 2]
    ///  Triangle 1: [0, 2, 3]
    fn make_2d() -> SlfGeometry {
        let npd3 = 3;
        let nplan = 1;
        // 2 triangles × 3 nodes each
        let mesh = vec![0, 1, 2, 0, 2, 3];
        let ipob3 = vec![1, 1, 1, 1]; // all boundary points
        let points = SlfArray2D::Float {
            x: vec![0.0, 1.0, 1.0, 0.0],
            y: vec![0.0, 0.0, 1.0, 1.0],
        };
        SlfGeometry::new(points, ipob3, mesh, npd3, nplan)
    }

    /// Build a minimal 3D geometry (nplan = 2, triangular prisms: npd3 = 6).
    ///
    ///  2 layers × 4 points each = 8 points total
    ///  2 prisms (1 element per layer × 1 layer gap = 1 element, but we use 2
    ///  triangular base elements × (nplan-1) layers = 2 prisms)
    fn make_3d() -> SlfGeometry {
        let npd3 = 6;
        let nplan = 2;
        // 2 prisms × 6 nodes each: bottom layer [0,1,2] top layer [4,5,6]
        let mesh = vec![
            0, 1, 2, 4, 5, 6, // prism 0
            0, 2, 3, 4, 6, 7, // prism 1
        ];
        let ipob3 = vec![1, 1, 1, 1, 1, 1, 1, 1]; // 8 points
        let points = SlfArray2D::Float {
            x: vec![0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0],
            y: vec![0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0],
        };
        SlfGeometry::new(points, ipob3, mesh, npd3, nplan)
    }

    // -----------------------------------------------------------------------
    // new() / Default
    // -----------------------------------------------------------------------

    #[test]
    fn new_sets_boundaries_and_interfaces_to_zero() {
        let g = make_2d();
        assert_eq!(g.boundaries_count, 0);
        assert_eq!(g.interfaces_count, 0);
    }

    #[test]
    fn default_is_empty() {
        let g = SlfGeometry::default();
        assert_eq!(g.points_count(), 0);
        assert_eq!(g.elements_count(), 0);
    }

    // -----------------------------------------------------------------------
    // dimension
    // -----------------------------------------------------------------------

    #[test]
    fn dimension_is_2_when_nplan_is_1() {
        assert_eq!(make_2d().dimension(), 2);
    }

    #[test]
    fn dimension_is_3_when_nplan_is_2() {
        assert_eq!(make_3d().dimension(), 3);
    }

    // -----------------------------------------------------------------------
    // planes_cnt
    // -----------------------------------------------------------------------

    #[test]
    fn planes_cnt_is_1_for_2d() {
        assert_eq!(make_2d().planes_cnt(), 1);
    }

    #[test]
    fn planes_cnt_matches_nplan_for_3d() {
        assert_eq!(make_3d().planes_cnt(), 2);
    }

    // -----------------------------------------------------------------------
    // points_count / npoin3
    // -----------------------------------------------------------------------

    #[test]
    fn points_count_matches_point_array_len_2d() {
        assert_eq!(make_2d().points_count(), 4);
    }

    #[test]
    fn points_count_matches_point_array_len_3d() {
        assert_eq!(make_3d().points_count(), 8);
    }

    #[test]
    fn npoin3_is_alias_for_points_count() {
        let g = make_2d();
        assert_eq!(g.npoin3(), g.points_count());
    }

    // -----------------------------------------------------------------------
    // points_per_layer / npoin2
    // -----------------------------------------------------------------------

    #[test]
    fn points_per_layer_equals_points_count_in_2d() {
        let g = make_2d();
        assert_eq!(g.points_per_layer(), g.points_count());
    }

    #[test]
    fn points_per_layer_is_total_divided_by_nplan_in_3d() {
        let g = make_3d();
        // 8 total points / 2 planes = 4 points per layer
        assert_eq!(g.points_per_layer(), 4);
    }

    #[test]
    fn npoin2_is_alias_for_points_per_layer() {
        let g = make_3d();
        assert_eq!(g.npoin2(), g.points_per_layer());
    }

    // -----------------------------------------------------------------------
    // elements_count
    // -----------------------------------------------------------------------

    #[test]
    fn elements_count_matches_mesh_len_2d() {
        // mesh has 6 entries (2 triangles × 3), but elements_count returns raw mesh.len()
        assert_eq!(make_2d().elements_count(), 2);
    }

    #[test]
    fn elements_count_matches_mesh_len_3d() {
        // mesh has 12 entries (2 prisms × 6)
        assert_eq!(make_3d().elements_count(), 2);
    }

    // -----------------------------------------------------------------------
    // elements_per_layer / nelem2
    // -----------------------------------------------------------------------

    #[test]
    fn elements_per_layer_equals_elements_count_in_2d() {
        let g = make_2d();
        assert_eq!(g.elements_per_layer(), g.elements_count());
    }

    #[test]
    fn elements_per_layer_is_total_divided_by_nplan_minus_1_in_3d() {
        let g = make_3d();
        // 12 mesh entries / (2 planes - 1) = 12 per layer gap
        assert_eq!(g.elements_per_layer(), 2);
    }

    #[test]
    fn nelem2_is_alias_for_elements_per_layer() {
        let g = make_3d();
        assert_eq!(g.nelem2(), g.elements_per_layer());
    }

    // -----------------------------------------------------------------------
    // point_per_element / point_per_layer_element / npd2
    // -----------------------------------------------------------------------

    #[test]
    fn point_per_element_is_npd3_in_2d() {
        assert_eq!(make_2d().point_per_element(), 3);
    }

    #[test]
    fn point_per_element_is_npd3_in_3d() {
        assert_eq!(make_3d().point_per_element(), 6);
    }

    #[test]
    fn point_per_layer_element_equals_npd3_in_2d() {
        assert_eq!(make_2d().point_per_layer_element(), 3);
    }

    #[test]
    fn point_per_layer_element_is_half_npd3_in_3d() {
        // triangular prism: 6 nodes → 3 in 2-D
        assert_eq!(make_3d().point_per_layer_element(), 3);
    }

    #[test]
    fn npd2_is_alias_for_point_per_layer_element() {
        let g2d = make_2d();
        let g3d = make_3d();
        assert_eq!(g2d.npd2(), g2d.point_per_layer_element());
        assert_eq!(g3d.npd2(), g3d.point_per_layer_element());
    }

    // -----------------------------------------------------------------------
    // ikle2
    // -----------------------------------------------------------------------

    #[test]
    fn ikle2_layer_0_returns_some_in_2d() {
        assert!(make_2d().ikle2(0).is_some());
    }

    #[test]
    fn ikle2_layer_0_returns_correct_slice_in_2d() {
        let g = make_2d();
        // Only one layer: the full mesh
        assert_eq!(g.ikle2(0).unwrap(), &[0u32, 1, 2, 0, 2, 3]);
    }

    #[test]
    fn ikle2_out_of_bounds_returns_none_in_2d() {
        // nplan = 1, so layer 1 is out of bounds
        assert!(make_2d().ikle2(1).is_none());
    }

    #[test]
    fn ikle2_layer_0_returns_some_in_3d() {
        assert!(make_3d().ikle2(0).is_some());
    }

    #[test]
    fn ikle2_layer_1_returns_some_in_3d() {
        assert!(make_3d().ikle2(1).is_some());
    }

    #[test]
    fn ikle2_out_of_bounds_returns_none_in_3d() {
        // nplan = 2, so layer 2 is out of bounds
        assert!(make_3d().ikle2(2).is_none());
    }

    // -----------------------------------------------------------------------
    // ipob2
    // -----------------------------------------------------------------------

    #[test]
    fn ipob2_layer_0_returns_some_in_2d() {
        assert!(make_2d().ipob2(0).is_some());
    }

    #[test]
    fn ipob2_out_of_bounds_returns_none_in_2d() {
        assert!(make_2d().ipob2(1).is_none());
    }

    #[test]
    fn ipob2_layer_0_returns_some_in_3d() {
        assert!(make_3d().ipob2(0).is_some());
    }

    #[test]
    fn ipob2_out_of_bounds_returns_none_in_3d() {
        assert!(make_3d().ipob2(2).is_none());
    }
}
