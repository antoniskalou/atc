use crate::geom::Point;

#[derive(Clone, Debug)]
pub struct LatLong {
    pub lat: dms_coordinates::DMS,
    pub long: dms_coordinates::DMS,
}

impl LatLong {
    // pub fn from_vector(origin: Point, v: Point) -> Self {
    //     // simple implementation, needs conversion to/from game world
    //     Self {
    //         lat: origin.x + v.x,
    //         long: origin.y + v.y,
    //     }
    // }

    pub fn to_game_world(&self, origin: LatLong) -> Point {
        Point { x: 0.0, y: 0.0 } 
    }
}
