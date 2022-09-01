use crate::geom::Point;

#[derive(Clone, Debug, PartialEq)]
pub enum CoordDirection {
    N,
    E,
    S,
    W,
}

// pub struct Coord(f32);

#[derive(Clone, Debug)]
pub struct Coord {
    pub degrees: i8,
    pub minutes: i8,
    pub seconds: i8,
    pub direction: CoordDirection,
}

#[derive(Clone, Debug)]
pub struct LatLong {
    pub lat: Coord,
    pub long: Coord,
}

impl LatLong {
    // pub fn from_vector(origin: Point, v: Point) -> Self {
    //     // simple implementation, needs conversion to/from game world
    //     Self {
    //         lat: origin.x + v.x,
    //         long: origin.y + v.y,
    //     }
    // }
}
