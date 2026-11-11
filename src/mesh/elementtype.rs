//! # Definition of spacial discretization
//!

/// Errors that occurs in operations on [ElementType]
#[derive(Debug, PartialEq)]
pub enum ElementError {
    UnsupportedElementSize {
        point_per_element: usize,
        dimension: u32,
    },
    UnsupportedDimension {
        dimension: u32,
    },
    UnsupportedCellElement {
        target: ElementType,
    },
}

/// # Type of element for spacial discretization (also known as cell)
///
/// There are two type of elements. P0 and P1 elements.
/// It refer to the type of interpolation functions (shape functions) used to
/// represent physical variables (like water depth, velocity, or tracers)
/// across the mesh elements.
/// The primary difference lies in how variables are distributed and approximated
/// within each element:
///
/// ## P0 Elements (Discontinuous Constant)
///
/// * **Definition**: Variables are assumed to be constant over the entire element.
/// * **Node Location**: The values are typically stored at the center
///   (barycenter) of the element.
/// * **Continuity**: The values are discontinuous at the element boundaries.
///   If you cross the edge from one element to another, the value jumps abruptly.
/// * **Usage**: Mostly used in the Finite Volume (FV) solvers of 2D
///   computation or for specific spatial discretizations where conservative,
///   shock-capturing properties are required (like highly torrential flows or dam-breaks).
///
/// ## P1 Elements (Continuous Linear)
///
/// * **Definition**: Variables vary linearly across the element.
/// * **Node Location**: The values are stored at the vertices (corners) of the
///   element.
/// * **Continuity**: The values are continuous across element boundaries. At
///   any shared edge or node, adjacent triangles agree on the exact same
///   value, though the slope (derivative) changes.
/// * **Usage**: This is the standard, traditional choice for the Finite
///   Element (FE) solvers in TELEMAC-2D and TELEMAC-3D. It provides a
///   smooth representation of the free surface and velocity fields.
///
/// ## P2 Elements (Piecewise Quadratic)
///
/// * **Definition**: Variable is approximated using a quadratic polynomial
///   (curved variation).
/// * **Node Location**: The values are stored at the three vertices plus the
///   midpoints of the three edges (6 nodes total per triangle).
/// * **Usage**: It provides much higher accuracy and captures complex,
///   non-linear variations or steep gradients more smoothly. However, because
///   it significantly increases the number of equations to solve, it
///   drastically increases CPU time and memory usage. It is typically reserved
///   for highly complex flow phenomena or specific turbulence modeling.
///
/// | Element Type | Polynomial Order   | Node Locations per Triangle | Degrees of Freedom | Accuracy & Smoothness                  | Computational Cost |
/// |--------------|--------------------|-----------------------------|--------------------|----------------------------------------|--------------------|
/// | P0           | Constant (Order 0) | Element center (barycenter) | 1 per element      | Low (discontinuous at boundaries)      | Very Low           |
/// | P1           | Linear (Order 1)   | Element vertices            | 3 per element      | Standard (continuous, sharp gradients) | Moderate (Default) |
/// | P2           | Quadratic (Order 2)| Vertices + Mid-edges        | 6 per element      | High (smooth curves, highly accurate)  | High               |
///

#[derive(Clone, Debug, Copy, PartialEq)]
pub enum ElementType {
    Null = -1,

    /// P0 (constant) element in dimension 1
    Point = 0,

    /// P1 (1st order) element in dimension 1
    PointBoundary = 1,

    /// P2 (2nd order) element in dimension 1
    Edge = 2,

    /// P0 (constant) element on triangles
    TriangleP0 = 10,

    /// P1 (1st order) element on triangles
    TriangleP1 = 11,

    /// quasi-bubble element on triangles,
    ///
    /// It is also called a 4-node triangle.
    ///
    /// A quasi-bubble triangle has 4 nodes: 3 nodes at each corner and a 4th
    ///  virtual one right at the center (the barycenter) of the triangle.
    /// It may increase mathematical stability and prevent numerical oscillations.
    ///
    /// ## Interpolation
    ///
    /// The interpolation functions within the element remain linear. The
    /// mathematical formulation effectively subdivides the main triangle
    /// into three smaller inner triangles meeting at this central node.
    ///
    /// ## The "Bubble" Concept
    ///
    /// In finite element mathematics, a "bubble function" is a polynomial
    /// that vanishes entirely along the boundaries of the element but
    /// peaks in the interior. This adds a localized degree of freedom
    /// to the equations.
    QuasiBubbleTriangle = 12,

    /// P1-Iso-P1 triangle
    ///
    /// a specific type of space discretization element used in the
    /// Finite Element Method to solve the Shallow Water (Saint-Venant)
    /// equations.
    ///
    /// It represents a linear velocity / linear water depth combination
    /// defined on a sub-divided mesh.
    TriangleP1IsoP1 = 13,

    /// P2 (2nd order) element on triangles
    TriangleP2 = 14,

    /// P0 (constant) element on quadrilaterals (3D: special case)
    QuadrangleP0 = 20,

    /// P1 (1st order) element on quadrilaterals (3d: special case)
    QuadrangleP1 = 21,
    QuadrangleP2 = 22,
    QuadrangleP4 = 24,

    /// T0 (constant) element on tetrahedrons
    TetrahedronP0 = 30,

    /// T1 (1st order) element on tetrahedrons
    TetrahedronP1 = 31,

    /// P0 (constant) element on prisms
    PrismP0 = 40,

    /// P1 (1st order) element on prisms
    PrismP1 = 41,

    /// Prisms split in T0 tetrahedrons
    PrismsSplitIntoTetrahedronP0 = 50,

    /// Prisms split in T0 tetrahedrons
    PrismsSplitIntoTetrahedronP1 = 51,
    EdgeBoundary = 55,

    /// P0 triangles on lateral side of 3D prism mesh
    /// (prism mesh split in tetrahedrons)
    TriangleBoundaryP0 = 60,

    /// P1 triangles on lateral side of 3D prism mesh
    /// (prism mesh split in tetrahedrons)
    TriangleBoundaryP1 = 61,

    /// Q0 quadrilaterals on lateral side of 3D prism meshes
    QuadrangleBoundaryP0 = 70,

    /// Q1 quadrilaterals on lateral side of 3D prism meshes
    QuadrangleBoundaryP1 = 71,

    /// P0 boundary triangles for tetrahedrons in an unstructured 3D mesh
    Triangle3DBoundaryP0 = 80,

    /// P1 boundary triangles for tetrahedrons in an unstructured 3D mesh
    Triangle3DBoundaryP1 = 81,
}

pub enum BoundaryLocation {
    /// Boundary is at the top or the bottom of the mesh
    HorizontalPlan,

    /// Boundary is at the lateral/vertical side of the mesh
    LateralPlan,
}

pub fn identify_element_type(
    dimension: u32,
    point_per_element: usize,
) -> Result<ElementType, ElementError> {
    match dimension {
        2 => match point_per_element {
            3 => Ok(ElementType::TriangleP0),
            4 => Ok(ElementType::TetrahedronP0),
            _ => Err(ElementError::UnsupportedElementSize {
                point_per_element,
                dimension,
            }),
        },
        3 => match point_per_element {
            4 => Ok(ElementType::TetrahedronP0),
            6 => Ok(ElementType::PrismP0),
            _ => Err(ElementError::UnsupportedElementSize {
                point_per_element,
                dimension,
            }),
        },
        _ => Err(ElementError::UnsupportedDimension { dimension }),
    }
}

impl ElementType {
    /// Dimension of an element
    ///
    /// Return an integer between 0 (for null) to 3 (for prism)
    ///
    /// # Example
    ///
    /// ```
    /// use free_surface::mesh::elementtype::ElementType;
    ///
    /// let elm_type = ElementType::TriangleP0;
    /// assert_eq!(elm_type.dimension(), 2);
    /// ```
    pub fn dimension(&self) -> u8 {
        match self {
            ElementType::Null => 0,
            //FIXME: Is a point of dimension 1?? It does not sound mathematically correct
            ElementType::Point | ElementType::PointBoundary | ElementType::Edge => 1,
            // FIXME: not clear in telemac source code what is the dimension of EdgeBoundary
            ElementType::EdgeBoundary => 1,
            ElementType::TriangleP0
            | ElementType::TriangleP1
            | ElementType::QuasiBubbleTriangle
            | ElementType::TriangleP1IsoP1
            | ElementType::TriangleP2
            | ElementType::QuadrangleP0
            | ElementType::QuadrangleP1
            | ElementType::QuadrangleP2
            | ElementType::QuadrangleP4
            | ElementType::TriangleBoundaryP0
            | ElementType::TriangleBoundaryP1
            | ElementType::QuadrangleBoundaryP0
            | ElementType::QuadrangleBoundaryP1
            | ElementType::Triangle3DBoundaryP0
            | ElementType::Triangle3DBoundaryP1 => 2,
            ElementType::TetrahedronP0
            | ElementType::TetrahedronP1
            | ElementType::PrismP0
            | ElementType::PrismP1
            | ElementType::PrismsSplitIntoTetrahedronP0
            | ElementType::PrismsSplitIntoTetrahedronP1 => 3,
        }
    }

    /// How many node in the element
    pub fn node(&self) -> u8 {
        match self {
            ElementType::Null => 0,
            ElementType::Point => 1,
            ElementType::PointBoundary => 2,
            ElementType::Edge => 3,                                 // ???
            ElementType::TriangleP0 => 1,                           // 1 at barycenter
            ElementType::TriangleP1 => 3,                           // 3 vertices
            ElementType::QuasiBubbleTriangle => 3 + 1,              // 3 vertices + barycenter
            ElementType::TriangleP1IsoP1 => 3 + 3,                  // 3 vertices + 3 at mid-edges
            ElementType::TriangleP2 => 3 + 3,                       // 3 vertices + 3 at mid-edges
            ElementType::QuadrangleP0 => 1,                         // 1 at barycenter
            ElementType::QuadrangleP1 => 4,                         // 4 vertices
            ElementType::QuadrangleP2 => 4 + 4,                     // 4 vertices + 4 at mid-edges
            ElementType::QuadrangleP4 => 4 + 4,                     // 4 vertices + 4 at mid-edges
            ElementType::TetrahedronP0 => 1,                        // 1 at barycenter
            ElementType::TetrahedronP1 => 4,                        // 4 vertices
            ElementType::PrismP0 => 1,                              // 1 at barycenter
            ElementType::PrismP1 => 6,                              // 6 vertices (TBC)
            ElementType::PrismsSplitIntoTetrahedronP0 => 1,         // 1 at barycenter
            ElementType::PrismsSplitIntoTetrahedronP1 => 4,         // 4 vertices
            ElementType::EdgeBoundary => panic!("not yet defined"), // To be defined
            ElementType::TriangleBoundaryP0 => 1,                   // 1 at barycenter
            ElementType::TriangleBoundaryP1 => 3,                   // 3 vertices
            ElementType::QuadrangleBoundaryP0 => 1,                 // 1 at barycenter
            ElementType::QuadrangleBoundaryP1 => 4,                 // 4 vertices
            ElementType::Triangle3DBoundaryP0 => 1,                 // 1 at barycenter
            ElementType::Triangle3DBoundaryP1 => 3,                 // 3 vertices
        }
    }

    /// Number of vertices per element
    pub fn vertices(&self) -> u8 {
        match self {
            ElementType::Null => 0,
            ElementType::Point => 1,
            ElementType::PointBoundary => 1,
            ElementType::Edge => panic!("not yet defined"),
            ElementType::TriangleP0 => 3,
            ElementType::TriangleP1 => 3,
            ElementType::QuasiBubbleTriangle => 3,
            ElementType::TriangleP1IsoP1 => 3,
            ElementType::TriangleP2 => 3,
            ElementType::QuadrangleP0 => 4,
            ElementType::QuadrangleP1 => 4,
            ElementType::QuadrangleP2 => 4,
            ElementType::QuadrangleP4 => 4,
            ElementType::TetrahedronP0 => 4,
            ElementType::TetrahedronP1 => 4,
            ElementType::PrismP0 => 5,
            ElementType::PrismP1 => 5,
            ElementType::PrismsSplitIntoTetrahedronP0 => 4,
            ElementType::PrismsSplitIntoTetrahedronP1 => 4,
            ElementType::EdgeBoundary => panic!("not yet defined"),
            ElementType::TriangleBoundaryP0 => 3,
            ElementType::TriangleBoundaryP1 => 3,
            ElementType::QuadrangleBoundaryP0 => 4,
            ElementType::QuadrangleBoundaryP1 => 4,
            ElementType::Triangle3DBoundaryP0 => 3,
            ElementType::Triangle3DBoundaryP1 => 3,
        }
    }

    /// Give the boundary [ElementType] for a given [ElementType] in the mesh
    ///
    /// When an element has a different shape for the top/bottom side and lateral side,
    /// the selection is done by `side` parameter.
    ///
    /// ```
    /// use free_surface::mesh::elementtype::{ElementType, BoundaryLocation};
    ///
    /// // A prism is composed by 3 quadrangles on lateral side and 2 triangles on top and
    /// // bottom side
    /// let cell_type = ElementType::PrismP0;
    ///
    /// assert_eq!(cell_type.boundary_type(BoundaryLocation::HorizontalPlan), Ok(ElementType::TriangleP0));
    /// assert_eq!(cell_type.boundary_type(BoundaryLocation::LateralPlan), Ok(ElementType::QuadrangleBoundaryP0));
    /// ```
    ///
    /// # Errors
    ///
    /// This function fails with [ElementError::UnsupportedCellElement] if called
    /// on a [ElementType] that cannot be used as a cell of a mesh
    pub fn boundary_type(&self, side: BoundaryLocation) -> Result<ElementType, ElementError> {
        match self {
            ElementType::TriangleP0 | ElementType::QuadrangleP0 => Ok(ElementType::Point),
            ElementType::TriangleP1
            | ElementType::QuasiBubbleTriangle
            | ElementType::QuadrangleP1 => Ok(ElementType::PointBoundary),
            ElementType::TriangleP1IsoP1 => Ok(ElementType::Edge),
            ElementType::TetrahedronP0 => Ok(ElementType::Triangle3DBoundaryP0),
            ElementType::TetrahedronP1 => Ok(ElementType::Triangle3DBoundaryP1),
            ElementType::PrismsSplitIntoTetrahedronP1 => match side {
                BoundaryLocation::HorizontalPlan => Ok(ElementType::TriangleP1),
                BoundaryLocation::LateralPlan => Ok(ElementType::TriangleBoundaryP1),
            },
            ElementType::PrismsSplitIntoTetrahedronP0 => match side {
                BoundaryLocation::HorizontalPlan => Ok(ElementType::TriangleP0),
                BoundaryLocation::LateralPlan => Ok(ElementType::TriangleBoundaryP0),
            },
            ElementType::PrismP0 => match side {
                BoundaryLocation::HorizontalPlan => Ok(ElementType::TriangleP0),
                BoundaryLocation::LateralPlan => Ok(ElementType::QuadrangleBoundaryP0),
            },
            ElementType::PrismP1 => match side {
                BoundaryLocation::HorizontalPlan => Ok(ElementType::TriangleP1),
                BoundaryLocation::LateralPlan => Ok(ElementType::QuadrangleBoundaryP1),
            },
            other => Err(ElementError::UnsupportedCellElement { target: *other }),
        }
    }
}
