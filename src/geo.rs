use crate::geom::Point;
use dms_coordinates::DMS;

const MEAN_EARTH_RADIUS: f64 = 6371008.8; 

#[derive(Clone, Debug)]
pub struct LatLon {
    pub lat: DMS,
    pub lon: DMS,
}

impl LatLon {
    // pub fn from_vector(origin: Point, v: Point) -> Self {
    //     // simple implementation, needs conversion to/from game world
    //     Self {
    //         lat: origin.x + v.x,
    //         lon: origin.y + v.y,
    //     }
    // }

    // TODO: test, not sure if implementation is correct
    pub fn distance(&self, other: LatLon) -> (f64, f64) {
        let x = dms_coordinates::projected_distance(
            (other.lat.to_ddeg_angle(), 0.0),
            (self.lat.to_ddeg_angle(), 0.0),
        );
        let y = dms_coordinates::projected_distance(
            (0.0, other.lon.to_ddeg_angle()),
            (0.0, self.lat.to_ddeg_angle()),
        );

        (x, y)
    }

    pub fn to_game_world(&self, origin: LatLon) -> Point {
        let (x, y) = self.distance(origin);
        Point {
            x: x as f32,
            y: y as f32,
        }
    }

    /// Return a new latitude/longitude offset by a distance in meters and a bearing
    /// in degrees.
    /// 
    /// algorithm from http://edwilliams.org/avform147.htm#LL and
    /// https://docs.rs/geo/0.14.2/src/geo/algorithm/haversine_destination.rs.html#33
    pub fn haversine_destination(&self, distance: f64, bearing: f64) -> LatLon {
        let center_lat = self.lat.to_radians();
        let center_lon = self.lon.to_radians();
        let bearing_rad = bearing.to_radians();

        let rad = distance / MEAN_EARTH_RADIUS;

        let lat = {
            center_lat.sin() * rad.cos() + center_lat.cos() * rad.sin() * bearing_rad.cos()
        }
        .asin();

        let lon = { bearing_rad.sin() * rad.sin() * center_lat.cos() }
            .atan2(rad.cos() - center_lat.sin() * lat.sin())
            + center_lon;

        Self {
            lat: DMS::from_ddeg_latitude(lat.to_degrees()),
            lon: DMS::from_ddeg_longitude(lon.to_degrees()),
        }
    }
}

#[cfg(test)]
mod test {
    use std::io::Write;

    use super::*;

    const LAX: LatLon = LatLon {
        lat: DMS {
            degrees: 33,
            minutes: 57,
            seconds: 0.0,
            cardinal: Some(dms_coordinates::Cardinal::North),
        },
        lon: DMS {
            degrees: 118,
            minutes: 24,
            seconds: 0.0,
            cardinal: Some(dms_coordinates::Cardinal::West),
        },
    };

    #[test]
    fn test_latlon_haversine_destination() {
        let nm2km = 1.852;
        let distance = (100.0 * nm2km) * 1000.0;
        let dest = LAX.haversine_destination(distance, 66f64);
        assert_eq!(34, dest.lat.degrees);
        assert_eq!(36, dest.lat.minutes);
        assert_eq!(116, dest.lon.degrees);
        assert_eq!(33, dest.lon.minutes);
    }
}
