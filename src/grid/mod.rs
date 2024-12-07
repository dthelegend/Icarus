pub trait Grid {
    type Direction;
    type Coordinate;
}
pub struct Path<GridT: Grid>(Vec<GridT::Direction>);

mod axial_hex;
