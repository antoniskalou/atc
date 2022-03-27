use crate::atc::{AtcReply, AtcRequest};
use crate::command::AtcCommand;
use crate::geom::*;
use ggez::{
    graphics::{self, Color},
    Context, GameResult,
};

#[derive(Clone, Debug)]
pub struct AircraftDefinition {
    max_speed: u32,
    min_speed: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AircraftStatus {
    Taxi,
    Takeoff,
    Landing,
    Landed,
    Flight,
}

// only encodes flight callsigns, not aircraft
#[derive(Clone, Debug)]
pub struct Callsign {
    pub name: String,
    pub code: String,
    pub number: String,
}

impl Callsign {
    pub fn coded(&self) -> String {
        format!("{}{}", self.code, self.number)
    }

    pub fn spoken(&self) -> String {
        format!("{} {}", self.name, self.number)
    }

    pub fn from_string(s: String) -> Option<Self> {
        let s = s.to_uppercase();

        if s.len() > 3 {
            // TODO: Check number is actually valid
            let (code, number) = s.split_at(3);
            Some(Self {
                name: String::from(""), // TODO: fetch from DB
                code: code.to_string(),
                number: number.to_string(),
            })
        } else {
            None
        }
    }
}

impl std::fmt::Display for Callsign {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.coded())
    }
}

impl PartialEq for Callsign {
    fn eq(&self, other: &Self) -> bool {
        self.code == other.code && self.number == other.number
    }
}

#[derive(Clone, Debug)]
pub struct AircraftParameter {
    intended: f32,
    // FIXME: hide
    pub current: f32,
    lerp: Option<Lerp>,
}

impl AircraftParameter {
    pub fn new(current: f32) -> Self {
        Self {
            current: current,
            intended: current,
            lerp: None,
        }
    }

    /// duration is time per single value
    fn change(&mut self, intended: f32, duration: f32) {
        if self.intended != intended {
            self.intended = intended;

            let initial_diff = self.intended - self.current;
            self.lerp = Some(Lerp::new(
                self.current,
                self.intended,
                initial_diff.abs() * duration
            ));
        }
    }

    pub fn current(&mut self, dt: f32) -> f32 {
        if let Some(lerp) = self.lerp.as_mut().filter(|x| !x.is_finished()) {
            self.current = lerp.update(dt);
        } 
        self.current
    }
}

#[derive(Clone, Debug)]
pub struct Aircraft {
    pub position: Point,
    pub callsign: Callsign,
    /// bearing
    // FIXME: need to call current to continue, its opaque to caller
    pub heading: AircraftParameter,
    /// feet
    pub altitude: AircraftParameter,
    /// knots
    pub speed: AircraftParameter,
    pub status: AircraftStatus,
    pub cleared_to_land: bool,
}

impl Aircraft {
    pub fn change_heading(&mut self, new_course: i32) {
        // time for 1 degree change
        let duration = 0.1;
        let course = if new_course < 0 {
            360
        } else if new_course > 360 {
            0
        } else {
            new_course
        };
        self.heading.change(course as f32, duration);
    }

    pub fn change_altitude(&mut self, new_altitude: u32) {
        // seconds per 1000 feet
        let duration = 30.0 / 1000.0;
        self.altitude.change(new_altitude as f32, duration);
    }

    pub fn change_speed(&mut self, new_speed: u32) {
        // time for 1kt change
        let duration = 1.0;
        // TODO: depends on aircraft type
        self.speed.change(new_speed.clamp(150, 250) as f32, duration);
    }

    pub fn is_localizer_captured(&self, localizer: &ILS) -> bool {
        is_point_in_triangle(self.position, localizer.as_triangle())
            && self.altitude.current as u32 <= localizer.altitude(self.position)
    }

    pub fn is_grounded(&self) -> bool {
        self.status == AircraftStatus::Taxi || self.status == AircraftStatus::Landed
    }

    pub fn cleared_to_land(&self) -> bool {
        self.cleared_to_land
    }

    pub fn command(&mut self, cmd: AtcRequest) -> AtcReply {
        use AtcCommand::*;
        match cmd.0 {
            ChangeHeading(heading) => {
                self.change_heading(heading)
                // reply
                // TODO
            }
            ChangeAltitude(altitude) => self.change_altitude(altitude),
            ChangeSpeed(speed) => self.change_speed(speed),
            ClearedToLand(is_cleared) => {
                self.cleared_to_land = is_cleared;
            }
        }
        AtcReply(cmd.0)
    }
}

impl PartialEq for Aircraft {
    fn eq(&self, other: &Self) -> bool {
        self.callsign == other.callsign &&
            self.position == other.position
    }
}

pub fn aircraft_by_callsign(
    callsign: Callsign,
    aircraft: &Vec<Aircraft>,
) -> Option<(usize, &Aircraft)> {
    let idx = aircraft.iter().position(|a| a.callsign == callsign);
    idx.map(|i| (i, &aircraft[i]))
}

pub const ILS_LENGTH: f32 = 500.0;

#[derive(Clone, Debug)]
pub struct ILS {
    // position at end of the runway
    origin: Point,
    runway: Runway,
}

impl ILS {
    pub fn as_triangle(&self) -> Vec<Point> {
        let localizer = [
            self.origin,
            // 3 degree variance
            rotate_point(
                self.origin,
                Point {
                    x: self.origin.x,
                    y: self.origin.y + ILS_LENGTH,
                },
                3f32.to_radians(),
            ),
            rotate_point(
                self.origin,
                Point {
                    x: self.origin.x,
                    y: self.origin.y + ILS_LENGTH,
                },
                -3f32.to_radians(),
            ),
        ];

        rotate_points(
            self.origin,
            &localizer,
            (self.runway.heading as f32).to_radians(),
        )
    }

    pub fn distance(&self, position: Point) -> f32 {
        point_distance(position, self.origin)
    }

    pub fn altitude(&self, position: Point) -> u32 {
        let distance = self.distance(position);
        let expected_alt = self.runway.ils_max_altitude as f32 * (distance / ILS_LENGTH);
        // round to 1000
        let rounded_alt = (expected_alt / 1000.0).round() * 1000.0;
        rounded_alt as u32
    }
}

const RUNWAY_DOWNSCALE: f32 = 10.0;

#[derive(Clone, Debug)]
pub struct Runway {
    /// offset from airport
    pub offset: Point,
    /// bearing
    pub heading: u32,
    /// length in meters
    pub length: u32,
    /// width in meters
    pub width: u32,
    /// in feet
    pub ils_max_altitude: u32,
}

impl Runway {
    pub fn as_line(&self, origin: Point) -> Vec<Point> {
        rotate_points(
            origin,
            &[
                Point {
                    x: origin.x,
                    y: origin.y - (self.length as f32 / RUNWAY_DOWNSCALE / 2.0),
                },
                Point {
                    x: origin.x,
                    y: origin.y + (self.length as f32 / RUNWAY_DOWNSCALE / 2.0),
                },
            ],
            (self.heading as f32).to_radians(),
        )
    }

    pub fn ils(&self, origin: Point) -> ILS {
        let origin = Point {
            // rotated runway line points
            x: self.as_line(origin)[1].x,
            y: self.as_line(origin)[1].y,
        };
        // note, state not automatically updated
        ILS {
            origin,
            runway: self.clone(),
        }
    }

    // TODO
    pub fn has_landed(&self, origin: Point, aircraft: &Aircraft) -> bool {
        is_point_in_circle(aircraft.position, origin, 10.0)
    }

    pub fn as_mesh(
        &self,
        ctx: &mut Context,
        origin: Point,
        color: Color,
    ) -> GameResult<graphics::Mesh> {
        graphics::Mesh::new_line(
            ctx,
            &self.as_line(origin),
            self.width as f32 / RUNWAY_DOWNSCALE,
            color,
        )
    }
}

#[derive(Clone, Debug)]
pub struct Airport {
    pub position: Point,
    pub icao_code: String,
    pub takeoff_runways: Vec<Runway>,
    pub landing_runways: Vec<Runway>,
}

impl Airport {
    pub fn origin(&self, runway: &Runway) -> Point {
        Point {
            x: self.position.x + runway.offset.x,
            y: self.position.y + runway.offset.y,
        }
    }
}
