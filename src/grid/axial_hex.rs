use nalgebra::SMatrix;
use crate::grid::Grid;

// Axial Hex Grid implementation
pub struct AxialHexGrid<T, const size: usize>(SMatrix<T, size, size>);
pub enum AxialHexGridDirection {
    NorthWest,
    NorthEast,
    East,
    SouthEast,
    SouthWest,
    West,
}

impl<T, const size: usize> Grid for AxialHexGrid<T, size> {
    type Direction = AxialHexGridDirection;
    type Coordinate = (isize, isize);
}
