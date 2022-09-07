use std::fmt::Display;

/// Mean radius of the earth: 6.37 * 10^6 m
const MEAN_EARTH_RADIUS: f64 = 6371008.8;

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
            write!(f, "{}°{}'{:.2}\"{}", self.degrees, self.minutes, self.seconds, cardinal)
        } else {
            write!(f, "{}°{}'{:.2}\"", self.degrees, self.minutes, self.seconds)
        }
    }
}

#[derive(Clone, Debug)]
pub struct LatLon(nav_types::WGS84<f64>);

impl LatLon {
    pub fn from_dms(lat: DMS, lon: DMS) -> Self {
        Self(nav_types::WGS84::from_degrees_and_meters(
            lat.to_degrees(),
            lon.to_degrees(),
            0.0,
        ))
    }

    pub fn to_dms(&self) -> (DMS, DMS) {
        (
            DMS::from_degrees_latitude(self.0.latitude_degrees()),
            DMS::from_degrees_longitude(self.0.longitude_degrees()),
        )
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
    pub fn haversine_destination(&self, distance: f64, bearing: f64) -> LatLon {
        // TODO: consider using the vincenty formulae
        let center_lat = self.0.latitude_radians();
        let center_lon = self.0.longitude_radians();
        let bearing_rad = bearing.to_radians();

        let rad = distance / MEAN_EARTH_RADIUS;

        let lat =
            { center_lat.sin() * rad.cos() + center_lat.cos() * rad.sin() * bearing_rad.cos() }
                .asin();

        let lon = { bearing_rad.sin() * rad.sin() * center_lat.cos() }
            .atan2(rad.cos() - center_lat.sin() * lat.sin())
            + center_lon;

        Self(nav_types::WGS84::from_radians_and_meters(lat, lon, 0.0))
    }

    pub fn haversine_distance(&self, other: &LatLon) -> f64 {
        self.0.distance(&other.0)
    }

    pub fn vincenty_distance(&self, other: &LatLon) -> f64 {
        let ecef: nav_types::ECEF<f64> = self.0.into();
        let ecef_other: nav_types::ECEF<f64> = other.0.into();

        ecef.distance(&ecef_other)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use nav_types::WGS84;

    const NM2KM: f64 = 1.852;

    #[test]
    fn test_latlon_haversine_destination() {
        let lax = LatLon(nav_types::WGS84::from_degrees_and_meters(
            33.95, -118.4, 0.0,
        ));
        let distance = (100.0 * NM2KM) * 1000.0;
        let dest = lax.haversine_destination(distance, 66f64);
        assert_eq!(34.6, (dest.0.latitude_degrees() * 10.0).round() / 10.0);
        assert_eq!(-116.6, (dest.0.longitude_degrees() * 10.0).round() / 10.0);
        assert_eq!(distance.round(), lax.haversine_distance(&dest).round());
    }

    #[test]
    fn test_latlon_haversine_distance() {
        let lcph = LatLon(WGS84::from_degrees_and_meters(34.717778, 32.485556, 0.0));
        let lclk = LatLon(WGS84::from_degrees_and_meters(34.875, 33.624722, 0.0));

        assert_eq!(105_596.0, lcph.haversine_distance(&lclk).round());
    }

    #[test]
    fn test_latlon_vincenty_distance() {
        let lcph = LatLon(WGS84::from_degrees_and_meters(34.717778, 32.485556, 0.0));
        let lclk = LatLon(WGS84::from_degrees_and_meters(34.875, 33.624722, 0.0));

        assert_eq!(105_696.0, lcph.vincenty_distance(&lclk).round());
    }

    #[test]
    fn test_latlon_distance() {
        unimplemented!();
    }
}
