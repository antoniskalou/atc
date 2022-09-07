//! Angle representation in D°M'S" (sexagesimal format).
//! Supports arithmetics operation, up to double precision,
//! for easy navigation calculations.
use thiserror::Error;
use crate::cardinal::Cardinal;

/// Angle expressed as `D°M'S"`, 
/// in Degrees D°, Minutes M' and fractionnal
/// Seconds S" (double precision) with an optionnal Cardinal.
/// When a cardinal is associated to this angle,
/// we consider this angle represents either a Latitude
/// or a Longitude angle.
#[derive(PartialEq, Copy, Clone, Debug)]
pub struct DMS {
    /// Degrees D° 
    pub degrees: u16,
    /// Minutes M'
    pub minutes: u8,
    /// Seconds with fractionnal part S"
    pub seconds: f64,
    /// Optionnal cardinal associated to this angle
    pub cardinal: Option<Cardinal>,
}

#[derive(Error, Debug)]
pub enum OpsError {
    #[error("incompatible cardinals")]
    IncompatibleCardinals,
}

impl std::fmt::Display for DMS {
    fn fmt (&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if let Some(cardinal) = self.cardinal {
            write!(f, "{}°{}'{}\"{}",
                self.degrees, 
                self.minutes, 
                self.seconds,
                cardinal,
            )
        } else {
            write!(f, "{}°{}'{}\"", 
                self.degrees,
                self.minutes,
                self.seconds,
            )
        }
    }
}

impl Default for DMS {
    /// Builds null angle with no Cardinal associated to it
    fn default() -> Self { 
        Self {
            degrees: 0,
            minutes: 0,
            seconds: 0.0_f64,
            cardinal: None,
        }
    }
}

impl DMS {
    /// Builds `D°M'S"` angle, from given D°, M', S" values.
    /// This method allows overflow, it will wrapp values to correct range
    /// itself.
    pub fn new (degrees: u16, minutes: u8, seconds: f64, cardinal: Option<Cardinal>) -> DMS { 
        let d =  Self::from_seconds(
            degrees as f64 * 3600.0 
                + minutes as f64 * 60.0
                    + seconds);
        if let Some(cardinal) = cardinal {
            d.with_cardinal(cardinal)
        } else {
            d
        }
    }

    /// Builds `D°M'S"` angle from total amount of seconds
    fn from_seconds (seconds: f64) -> Self {
        let degrees = (seconds / 3600.0).floor();
        let minutes = ((seconds - degrees * 3600.0) /60.0).floor();
        let integer = ((seconds - degrees * 3600.0 - minutes*60.0).floor() as u8)%60;
        Self {
            degrees: (degrees as u16)%360,
            minutes: minutes as u8,
            seconds: integer as f64 + seconds.fract(),
            cardinal: None,
        }
    }

    /// Returns same D°M'S" angle but attaches a cardinal to it.
    /// Useful to convert make this D°M'S" angle a Latitude or a
    /// Longitude.
    fn with_cardinal (&self, cardinal: Cardinal) -> Self {
        Self {
            degrees: self.degrees,
            minutes: self.minutes,
            seconds: self.seconds,
            cardinal: Some(cardinal),
        }
    }

    /// Builds D°M'S" angle from given angle expressed in 
    /// decimal degrees, with no cardinal associated to returned value
    pub fn from_ddeg_angle (angle: f64) -> Self {
        let degrees = angle.abs().floor();
        let minutes = ((angle.abs() - degrees) * 60.0).floor();
        let seconds = (angle.abs() - degrees - minutes/60.0_f64) * 3600.0_f64;
        Self {
            degrees: degrees as u16,
            minutes: minutes as u8,
            seconds,
            cardinal: None,
        }
    }

    /// Builds Latitude angle, expressed in D°M'S", from
    /// given angle expressed in decimal degrees
    pub fn from_ddeg_latitude (angle: f64) -> Self {
        let degrees = angle.abs().floor();
        let minutes = ((angle.abs() - degrees) * 60.0).floor();
        let seconds = (angle.abs() - degrees - minutes/60.0_f64) * 3600.0_f64;
        let cardinal = if angle < 0.0 {
            Cardinal::South
        } else {
            Cardinal::North
        };
        Self {
            degrees: (degrees as u16)%90,
            minutes: minutes as u8,
            seconds,
            cardinal: Some(cardinal),
        }
    }

    /// Builds Longitude angle, expressed in D°M'S",
    /// from given angle expressed in decimal degrees
    pub fn from_ddeg_longitude (angle: f64) -> Self {
        let degrees = angle.abs().floor();
        let minutes = (angle.abs() - degrees) * 60.0;
        let seconds = (angle.abs() - degrees - minutes/60.0_f64) * 3600.0_f64;
        let cardinal = if angle < 0.0 {
            Cardinal::West
        } else {
            Cardinal::East
        };
        Self {
            degrees: (degrees as u16)%180,
            minutes: minutes as u8,
            seconds,
            cardinal: Some(cardinal),
        }
    }

    /// Returns Self expressed in decimal degrees
    /// If no cardinal is associated, returned angle strictly > 0.
    pub fn to_ddeg_angle (&self) -> f64 {
        let d = self.degrees as f64
            + self.minutes as f64 / 60.0_f64
                + self.seconds as f64 / 3600.0_f64;
        match self.cardinal {
            Some(cardinal) => {
                if cardinal.is_southern() || cardinal.is_western() {
                    -d
                } else {
                    d
                }
            },
            None => d,
        }
    }

    /// Converts self to radians
    pub fn to_radians (&self) -> f64 {
        self.to_ddeg_angle() / 180.0 * std::f64::consts::PI 
    }
}
