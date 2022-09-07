use geographiclib_rs::{DirectGeodesic, Geodesic, InverseGeodesic};
use std::fmt::Display;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Cardinal {
    North,
    South,
    East,
    West,
}

impl Display for Cardinal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Cardinal::North => "N",
            Cardinal::South => "S",
            Cardinal::East => "E",
            Cardinal::West => "W",
        };
        write!(f, "{}", s)
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct DMS {
    pub degrees: u16,
    pub minutes: u8,
    pub seconds: f64,
    pub cardinal: Option<Cardinal>,
}

impl DMS {
    pub fn new(degrees: u16, minutes: u8, seconds: f64, cardinal: Cardinal) -> Self {
        Self {
            degrees,
            minutes,
            seconds,
            cardinal: Some(cardinal),
        }
    }

    pub fn from_degrees(degrees: f64) -> Self {
        let d = degrees.abs().floor();
        let m = ((degrees.abs() - d) * 60.0).floor();
        let s = (degrees.abs() - d - m / 60.0) * 3600.0;

        Self {
            degrees: d as u16,
            minutes: m as u8,
            seconds: s,
            cardinal: None,
        }
    }

    pub fn from_degrees_latitude(lat: f64) -> Self {
        let mut d = Self::from_degrees(lat);

        if lat < 0.0 {
            d.cardinal = Some(Cardinal::South);
        } else {
            d.cardinal = Some(Cardinal::North);
        }

        d
    }

    pub fn from_degrees_longitude(lon: f64) -> Self {
        let mut d = Self::from_degrees(lon);

        if lon < 0.0 {
            d.cardinal = Some(Cardinal::West);
        } else {
            d.cardinal = Some(Cardinal::East);
        }

        d
    }

    pub fn to_degrees(&self) -> f64 {
        let d = self.degrees as f64 + self.minutes as f64 / 60.0 + self.seconds / 3600.0;

        self.cardinal
            .map(|cardinal| {
                if cardinal == Cardinal::South || cardinal == Cardinal::West {
                    -d
                } else {
                    d
                }
            })
            .unwrap_or(d)
    }
}

impl Display for DMS {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(cardinal) = self.cardinal {
            write!(
                f,
                "{}°{}'{:.2}\"{}",
                self.degrees, self.minutes, self.seconds, cardinal
            )
        } else {
            write!(f, "{}°{}'{:.2}\"", self.degrees, self.minutes, self.seconds)
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct LatLon {
    lat: f64,
    lon: f64,
}

impl LatLon {
    pub fn new(lat: f64, lon: f64) -> Self {
        Self { lat, lon }
    }

    pub fn from_dms(lat: DMS, lon: DMS) -> Self {
        Self {
            lat: lat.to_degrees(),
            lon: lon.to_degrees(),
        }
    }

    pub fn to_dms(&self) -> (DMS, DMS) {
        (
            DMS::from_degrees_latitude(self.latitude()),
            DMS::from_degrees_longitude(self.longitude()),
        )
    }

    pub fn latitude(&self) -> f64 {
        self.lat
    }

    pub fn longitude(&self) -> f64 {
        self.lon
    }

    // TODO: test, not sure if implementation is correct
    // pub fn distance(&self, other: LatLon) -> (f64, f64) {
    //     let x = dms_coordinates::projected_distance(
    //         (other.lat.to_ddeg_angle(), 0.0),
    //         (self.lat.to_ddeg_angle(), 0.0),
    //     );
    //     let y = dms_coordinates::projected_distance(
    //         (0.0, other.lon.to_ddeg_angle()),
    //         (0.0, self.lat.to_ddeg_angle()),
    //     );

    //     (x, y)
    // }

    // pub fn to_game_world(&self, origin: LatLon) -> Point {
    //     let (x, y) = self.distance(origin);
    //     Point {
    //         x: x as f32,
    //         y: y as f32,
    //     }
    // }

    /// Return a new latitude/longitude offset by a distance in meters and a bearing
    /// in degrees.
    ///
    /// algorithm from http://edwilliams.org/avform147.htm#LL and
    /// https://docs.rs/geo/0.14.2/src/geo/algorithm/haversine_destination.rs.html#33

    pub fn destination(&self, bearing: f64, distance: f64) -> LatLon {
        let g = Geodesic::wgs84();
        let (lat, lon) = g.direct(self.lat, self.lon, bearing, distance);
        Self { lat, lon }
    }

    pub fn distance(&self, other: &LatLon) -> f64 {
        let g = Geodesic::wgs84();
        g.inverse(self.lat, self.lon, other.lat, other.lon)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const NM2KM: f64 = 1.852;

    #[test]
    fn test_latlon_destination() {
        let lax = LatLon::new(33.95, -118.4);
        let distance = (100.0 * NM2KM) * 1000.0;
        let dest = lax.destination(66.0, distance);
        assert_eq!(34.6, (dest.latitude() * 10.0).round() / 10.0);
        assert_eq!(-116.6, (dest.longitude() * 10.0).round() / 10.0);
        assert_eq!(distance.round(), lax.distance(&dest).round());
    }

    #[test]
    fn test_latlon_distance() {
        let lcph = LatLon::new(34.717778, 32.485556);
        let lclk = LatLon::new(34.875, 33.624722);

        assert_eq!(105_698., lcph.distance(&lclk).round());
    }
}
