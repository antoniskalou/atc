use crate::atc::{AtcReply, AtcRequest};
use crate::camera::Camera;
use crate::command::AtcCommand;
use crate::geom::{self, *};
use crate::{math::*, units};
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

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TurnDirection {
    Left,
    Right,
}

impl std::fmt::Display for TurnDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Left => "left",
                Self::Right => "right",
            }
        )
    }
}

#[derive(Clone, Debug)]
pub struct HeadingParameter {
    intended: f32,
    // FIXME: hide
    pub current: f32,
    interpolator: Option<Interpolator>,
}

impl HeadingParameter {
    pub fn new(current: f32) -> Self {
        Self {
            current,
            intended: current,
            interpolator: None,
        }
    }

    fn change(&mut self, intended: f32, duration: f32) {
        if self.intended != intended {
            self.intended = intended;

            let initial_diff = short_angle_distance(self.intended, self.current).abs();
            let duration = initial_diff * duration;
            self.interpolator = Some(Interpolator::with_fn(
                self.current,
                self.intended,
                duration,
                angle_lerp,
            ));
        }
    }

    fn change_with_turn(&mut self, intended: f32, duration: f32, direction: TurnDirection) {
        if self.intended != intended {
            self.intended = intended;

            let initial_diff = short_angle_distance(self.intended, self.current);
            let should_flip = match direction {
                TurnDirection::Left => initial_diff < 0.0,
                TurnDirection::Right => initial_diff > 0.0,
            };

            let duration_fn = if should_flip {
                long_angle_distance
            } else {
                short_angle_distance
            };
            let duration = duration_fn(self.intended, self.current).abs() * duration;

            self.interpolator = Some(Interpolator::with_fn(
                self.current,
                self.intended,
                duration,
                if should_flip {
                    long_angle_lerp
                } else {
                    angle_lerp
                },
            ));
        }
    }

    pub fn current(&mut self, dt: f32) -> f32 {
        if let Some(interpolator) = self.interpolator.as_mut().filter(|x| !x.is_finished()) {
            self.current = interpolator.update(dt);
        }
        self.current
    }
}

#[derive(Clone, Debug)]
pub struct AircraftParameter {
    intended: f32,
    // FIXME: hide
    pub current: f32,
    interpolator: Option<Interpolator>,
}

impl AircraftParameter {
    pub fn new(current: f32) -> Self {
        Self {
            current: current,
            intended: current,
            interpolator: None,
        }
    }

    /// duration is time per single value
    fn change(&mut self, intended: f32, duration: f32) {
        if self.intended != intended {
            self.intended = intended;

            let initial_diff = self.intended - self.current;
            // let initial_diff = short_angle_distance(self.intended, self.current);
            let duration = initial_diff.abs() * duration;
            self.interpolator = Some(Interpolator::new(self.current, self.intended, duration));
        }
    }

    pub fn current(&mut self, dt: f32) -> f32 {
        if let Some(interpolator) = self.interpolator.as_mut().filter(|x| !x.is_finished()) {
            self.current = interpolator.update(dt);
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
    pub heading: HeadingParameter,
    /// feet
    pub altitude: AircraftParameter,
    /// knots
    pub speed: AircraftParameter,
    pub status: AircraftStatus,
    pub cleared_to_land: bool,
}

impl Aircraft {
    pub fn change_heading(&mut self, course: i32, direction: Option<TurnDirection>) {
        // time for 1 degree change
        let duration = 0.1;
        // FIXME: don't use clamp, use rem_euclid (maybe)
        let course = clamp(course, 0, 360) as f32;

        match direction {
            Some(direction) => self.heading.change_with_turn(course, duration, direction),
            None => self.heading.change(course, duration),
        }
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
        self.speed
            .change(clamp(new_speed, 150, 250) as f32, duration);
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
                self.change_heading(heading, None)
                // reply
                // TODO
            }
            ChangeHeadingWithTurnDirection(heading, direction) => {
                self.change_heading(heading, Some(direction))
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
        self.callsign == other.callsign && self.position == other.position
    }
}

pub fn aircraft_by_callsign(
    callsign: Callsign,
    aircraft: &Vec<Aircraft>,
) -> Option<(usize, &Aircraft)> {
    let idx = aircraft.iter().position(|a| a.callsign == callsign);
    idx.map(|i| (i, &aircraft[i]))
}

// 8nm
pub const ILS_LENGTH: f32 = 8. * units::NM_to_KM as f32 * 1000.;

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
            // ILS bearing is opposite of runway
            invert_bearing(self.runway.heading as f32).to_radians(),
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
                    y: origin.y - (self.length as f32 / 2.),
                },
                Point {
                    x: origin.x,
                    y: origin.y + (self.length as f32 / 2.),
                },
            ],
            (self.heading as f32).to_radians(),
        )
    }

    pub fn ils(&self, origin: Point) -> ILS {
        let origin = Point {
            // rotated runway line points
            x: self.as_line(origin)[0].x,
            y: self.as_line(origin)[0].y,
        };
        // note, state not automatically updated
        ILS {
            origin,
            runway: self.clone(),
        }
    }

    // TODO
    pub fn has_landed(&self, origin: Point, aircraft: &Aircraft) -> bool {
        is_point_in_circle(aircraft.position, origin, 500.0)
    }

    // FIXME: move me
    pub fn as_mesh(
        &self,
        ctx: &mut Context,
        origin: Point,
        color: Color,
        camera: &Camera,
    ) -> GameResult<graphics::Mesh> {
        let line = self
            .as_line(origin)
            // TODO: move elsewhere
            .iter()
            .map(|p| {
                camera.world_to_screen_coords(p.clone())
            })
            .collect::<Vec<Point>>();
        // TODO: move screen scale conversion
        graphics::Mesh::new_line(ctx, &line, self.width as f32 * camera.pixels_per_unit().x, color)
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
