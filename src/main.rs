mod geom;
mod tts;

use ggez::{
    event::{self, EventHandler, KeyCode, MouseButton},
    graphics::{self, Color},
    timer, Context, ContextBuilder, GameResult,
};
use crate::geom::*;

const TTS_ENABLED: bool = false;

#[derive(Debug)]
struct ATC {
    tts: Option<tts::TextToSpeech>,
}

impl ATC {
    fn new() -> ATC {
        ATC {
            tts: if TTS_ENABLED {
                Some(tts::TextToSpeech::new())
            } else { 
                None 
            },
        }
    }

    fn command(&mut self, aircraft: &mut Aircraft, cmd: ATCCommand) {
        // request
        println!("{}, ATC, {}", aircraft.callsign, cmd.as_string());
        if let Some(tts) = &mut self.tts {
            tts
                .say(format!("{}, ATC, {}", callsign_to_name(&aircraft.callsign), cmd.as_string()))
                .expect("failed to send tts message");
        }

        use ATCCommand::*;
        match cmd {
            ChangeHeading(heading) => {
                aircraft.change_heading(heading)
                // reply
                // TODO
            },
            ChangeAltitude(altitude) => aircraft.change_altitude(altitude),
            ChangeSpeed(speed) => aircraft.change_speed(speed),
            ClearedToLand(is_cleared) => {
                if is_cleared { 
                    aircraft.status = AircraftStatus::Landing;
                } else {
                    aircraft.status = AircraftStatus::Flight;
                }
            }
        }
    }
}

enum ATCCommand {
    ChangeHeading(i32),
    ChangeAltitude(u32),
    ChangeSpeed(u32),
    ClearedToLand(bool),
}

impl ATCCommand {
    fn from_string(s: String) -> Vec<ATCCommand> {
        let command_parts: Vec<&str> = s.split(' ').collect();
    
        let mut commands = Vec::new();
        let mut iter = command_parts.iter();
        loop {
            let cmd = iter.next()
                .map(|x| x.to_uppercase())
                .map(|x| match x.as_str() {
                    "LND" => ATCCommand::ClearedToLand(true), 
                    "HDG" => { 
                        // TODO: error handling
                        let hdg = iter.next().unwrap();
                        ATCCommand::ChangeHeading(hdg.parse::<i32>().unwrap())
                    },
                    "ALT" => {
                        let alt = iter.next().unwrap();
                        ATCCommand::ChangeAltitude(alt.parse::<u32>().unwrap())
                    },
                    "SPD" => {
                        let spd = iter.next().unwrap();
                        ATCCommand::ChangeSpeed(spd.parse::<u32>().unwrap())
                    }
                    _ => panic!("invalid command: {}", x)
                });

            match cmd {
                Some(cmd) => commands.push(cmd),
                None => break,
            }
        }
        commands
    }

    fn as_string(&self) -> String {
        use ATCCommand::*;
        match self {
            ChangeHeading(heading) => format!("heading to {}", heading),
            ChangeAltitude(alt) => format!("altitude to {}", alt),
            ChangeSpeed(speed) => format!("speed to {}", speed),
            ClearedToLand(cleared) => 
                String::from(if *cleared {
                    "cleared to land"
                } else { 
                    "clearance to land cancelled" 
                }),
            
        }
    }
}

struct ATCRequest(ATCCommand);
struct ATCReply(ATCCommand);

// FIXME: crappy way to get callsign
fn callsign_to_name(callsign: &String) -> String {
    let (icao_code, flight_number) = callsign.split_at(3);
    let spoken_callsign = 
        match icao_code {
            "TRA" => "Transavia",
            "CYP" => "Cyprus Airways",
            _other => {
                // TODO: use phonetic alphabet
                "unknown"
            },
        };

    format!("{} {}", spoken_callsign, flight_number)
}

#[derive(Clone, Debug)]
struct AircraftDefinition {
    max_speed: u32,
    min_speed: u32,
}

#[derive(Clone, Debug, PartialEq)]
enum AircraftStatus {
    Taxi,
    Takeoff,
    Landing,
    Landed,
    Flight,
}

#[derive(Clone, Debug)]
struct Aircraft {
    position: Point,
    callsign: String,
    // bearing
    heading: i32,
    /// feet
    altitude: u32,
    /// knots
    speed: u32,
    on_ils: Option<ILS>,
    status: AircraftStatus,
}

const AIRCRAFT_RADIUS: f32 = 4.0;
const AIRCRAFT_BOUNDING_RADIUS: f32 = AIRCRAFT_RADIUS * 5.0;

impl Aircraft {
    pub fn change_heading(&mut self, new_course: i32) {
        self.heading = if new_course < 0 {
            360
        } else if new_course > 360 {
            0
        } else {
            new_course
        };
    }

    pub fn change_altitude(&mut self, new_altitude: u32) {
        self.altitude = new_altitude.max(1000);
    }

    pub fn change_speed(&mut self, new_speed: u32) {
        // TODO: depends on aircraft type
        self.speed = new_speed.clamp(150, 250);
    }

    fn is_localizer_captured(&self, localizer: &ILS) -> bool {
        is_point_in_triangle(self.position, localizer.as_triangle()) &&
            self.altitude <= localizer.altitude(self.position)
    }

    fn is_grounded(&self) -> bool {
        self.status == AircraftStatus::Taxi || self.status == AircraftStatus::Landed
    }

    fn cleared_to_land(&self) -> bool {
        self.status == AircraftStatus::Landing
    }
}

const ILS_LENGTH: f32 = 300.0;

#[derive(Clone, Debug)]
struct ILS {
    // position at end of the runway
    origin: Point,
    runway: Runway,
}

impl ILS {
    fn as_triangle(&self) -> Vec<Point> {
        let localizer = [
            self.origin,
            // 3 degree variance
            rotate_point(
                self.origin,
                Point {
                    x: self.origin.x,
                    y: self.origin.y + ILS_LENGTH,
                },
                -3f32.to_radians(),
            ),
            rotate_point(
                self.origin,
                Point {
                    x: self.origin.x,
                    y: self.origin.y + ILS_LENGTH,
                },
                3f32.to_radians(),
            ),
        ];

        rotate_points(
            self.origin,
            &localizer,
            (self.runway.heading as f32).to_radians(),
        )
    }

    fn distance(&self, position: Point) -> f32 {
        point_distance(position, self.origin)
    }

    fn altitude(&self, position: Point) -> u32 {
        let distance = self.distance(position);
        let expected_alt = self.runway.ils_max_altitude as f32 * (distance / ILS_LENGTH);
        // round to 1000
        let rounded_alt = (expected_alt / 1000.0).round() * 1000.0;
        rounded_alt as u32
    }
}

const RUNWAY_DOWNSCALE: f32 = 10.0;

#[derive(Clone, Debug)]
struct Runway {
    /// offset from airport
    offset: Point,
    /// bearing
    heading: u32,
    /// length in meters
    length: u32,
    /// width in meters
    width: u32,
    /// in feet
    ils_max_altitude: u32,
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

    fn ils(&self, origin: Point) -> ILS {
        let origin = Point {
            // rotated runway line points
            x: self.as_line(origin)[1].x,
            y: self.as_line(origin)[1].y,
        };
        // note, state not automatically updated
        ILS { origin, runway: self.clone(), }
    }

    // TODO
    fn has_landed(&self, origin: Point, aircraft: &Aircraft) -> bool {
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
struct Airport {
    position: Point,
    icao_code: String,
    takeoff_runways: Vec<Runway>,
    landing_runways: Vec<Runway>,
}

impl Airport {
    fn origin(&self, runway: &Runway) -> Point {
        Point {
            x: self.position.x + runway.offset.x,
            y: self.position.y + runway.offset.y,
        }
    }
}

#[derive(Debug)]
struct Game {
    atc: ATC,
    airports: Vec<Airport>,
    selected_aircraft: usize,
    aircraft: Vec<Aircraft>,
}

impl Game {
    pub fn new(_ctx: &mut Context) -> Self {
        let runway_29 = Runway {
            offset: Point { x: 0.0, y: 0.0 },
            heading: 290,
            length: 1900,
            width: 35,
            ils_max_altitude: 2000,
        };

        Self {
            atc: ATC::new(),
            airports: vec![Airport {
                position: Point { x: 500.0, y: 550.0 },
                icao_code: "LCPH".into(),
                takeoff_runways: vec![runway_29.clone()],
                landing_runways: vec![runway_29.clone()],
            }],
            selected_aircraft: 0,
            aircraft: vec![
                Aircraft {
                    position: ggez::mint::Point2 { x: 250.0, y: 200.0 },
                    callsign: "CYP2202".into(),
                    heading: 90,
                    altitude: 6000,
                    speed: 250,
                    on_ils: None,
                    status: AircraftStatus::Flight,
                },
                Aircraft {
                    position: ggez::mint::Point2 { x: 500.0, y: 400.0 },
                    callsign: "TRA1112".into(),
                    heading: 180,
                    altitude: 12000,
                    speed: 220,
                    on_ils: None,
                    status: AircraftStatus::Flight,
                },
            ],
        }
    }
}

impl EventHandler<ggez::GameError> for Game {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        let dt = timer::delta(ctx);

        for mut aircraft in &mut self.aircraft {
            if !aircraft.is_grounded() {
                let speed_scale = 25.0;
                let speed_change = (aircraft.speed as f32 * dt.as_secs_f32()) / speed_scale;

                let heading = heading_to_vector(aircraft.heading);
                aircraft.position.x += speed_change * heading.x;
                aircraft.position.y += speed_change * heading.y;
            }

            // TODO: check if intercepting ILS
            if aircraft.cleared_to_land() {
                if let Some(ils) = &aircraft.on_ils {
                    let expected_alt = ils.altitude(aircraft.position);
                    aircraft.altitude = expected_alt;
                }

                // super inefficient
                for airport in &self.airports {
                    for runway in &airport.landing_runways {
                        let origin = airport.origin(runway);
                        let ils = runway.ils(origin);

                        if runway.has_landed(origin, aircraft) {
                            println!("Aircraft landed: {:?}", aircraft);
                            aircraft.on_ils = None;
                            aircraft.status = AircraftStatus::Landed;
                        } else if aircraft.is_localizer_captured(&ils) {
                            aircraft.heading = runway.heading as i32;
                            aircraft.on_ils = Some(ils);
                            aircraft.status = AircraftStatus::Landing;
                        }
                    }
                }
            }
        }

        // remove landed aircraft
        self.aircraft.retain(|a| !a.is_grounded());

        Ok(())
    }

    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        keycode: KeyCode,
        _keymod: event::KeyMods,
        _repeat: bool,
    ) {
        let aircraft = &mut self.aircraft[self.selected_aircraft];

        if keycode == KeyCode::A {
            let new_heading = aircraft.heading - 5;
            self.atc.command(aircraft, ATCCommand::ChangeHeading(new_heading));
        } else if keycode == KeyCode::D {
            let new_heading = aircraft.heading + 5;
            self.atc.command(aircraft, ATCCommand::ChangeHeading(new_heading));
        } else if keycode == KeyCode::S {
            let new_alt = aircraft.altitude - 1000;
            self.atc.command(aircraft, ATCCommand::ChangeAltitude(new_alt));
        } else if keycode == KeyCode::W {
            let new_alt = aircraft.altitude + 1000;
            self.atc.command(aircraft, ATCCommand::ChangeAltitude(new_alt));
        } else if keycode == KeyCode::F {
            let new_speed = aircraft.speed - 10;
            self.atc.command(aircraft, ATCCommand::ChangeSpeed(new_speed));
        } else if keycode == KeyCode::R {
            let new_speed = aircraft.speed + 10;
            self.atc.command(aircraft, ATCCommand::ChangeSpeed(new_speed));
        } else if keycode == KeyCode::L {
            self.atc.command(aircraft, ATCCommand::ClearedToLand(!aircraft.cleared_to_land()));
        } else if keycode == KeyCode::LBracket {
            self.selected_aircraft = (self.selected_aircraft as i32 - 1).max(0) as usize;
        } else if keycode == KeyCode::RBracket {
            self.selected_aircraft = (self.selected_aircraft + 1).min(self.aircraft.len() - 1);
        }
    }

    fn mouse_button_down_event(&mut self, _ctx: &mut Context, button: MouseButton, x: f32, y: f32) {
        // aircraft selection
        if button == MouseButton::Left {
            let click_pos = Point { x, y };

            for (i, aircraft) in self.aircraft.iter().enumerate() {
                if is_point_in_circle(click_pos, aircraft.position, AIRCRAFT_BOUNDING_RADIUS) {
                    self.selected_aircraft = i;
                    break;
                }
            }
        }
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, Color::BLACK);

        for airport in &self.airports {
            let icao_text = graphics::Text::new(airport.icao_code.clone());
            graphics::queue_text(ctx, &icao_text, Point { x: 0.0, y: 0.0 }, Some(Color::BLUE));
            graphics::draw_queued_text(
                ctx,
                graphics::DrawParam::new().dest(airport.position),
                None,
                graphics::FilterMode::Linear,
            )?;

            for runway in &airport.landing_runways {
                let origin = airport.origin(runway);
                let mesh = runway.as_mesh(ctx, origin, Color::RED)?;
                graphics::draw(ctx, &mesh, (Point { x: 0.0, y: 0.0 },))?;

                let ils = runway.ils(origin).as_triangle();
                let mesh = graphics::Mesh::new_polygon(
                    ctx,
                    graphics::DrawMode::stroke(2.0),
                    &ils,
                    Color::BLUE,
                )?;
                graphics::draw(ctx, &mesh, (Point { x: 0.0, y: 0.0 },))?;
            }
        }

        for aircraft in &self.aircraft {
            let aircraft_rect = graphics::Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::fill(),
                graphics::Rect::new(
                    aircraft.position.x - AIRCRAFT_RADIUS,
                    aircraft.position.y - AIRCRAFT_RADIUS,
                    AIRCRAFT_RADIUS * 2.0,
                    AIRCRAFT_RADIUS * 2.0,
                ),
                Color::GREEN,
            )?;

            graphics::draw(ctx, &aircraft_rect, (Point { x: 0.0, y: 0.0 },))?;

            let bounding_circle = graphics::Mesh::new_circle(
                ctx,
                graphics::DrawMode::stroke(2.0),
                aircraft.position,
                AIRCRAFT_BOUNDING_RADIUS,
                1.0,
                Color::GREEN,
            )?;

            graphics::draw(ctx, &bounding_circle, (Point { x: 0.0, y: 0.0 },))?;

            let callsign_text = graphics::Text::new(aircraft.callsign.clone());
            graphics::queue_text(
                ctx,
                &callsign_text,
                Point { x: -20.0, y: 30.0 },
                Some(Color::GREEN),
            );
            let heading_text = graphics::Text::new(format!("H{}", aircraft.heading));
            graphics::queue_text(
                ctx,
                &heading_text,
                Point { x: -20.0, y: 45.0 },
                Some(Color::GREEN),
            );
            let altitude_text = graphics::Text::new(format!("{}", aircraft.altitude));
            graphics::queue_text(
                ctx,
                &altitude_text,
                Point { x: 20.0, y: 45.0 },
                Some(Color::GREEN),
            );

            if aircraft.cleared_to_land() {
                let text = graphics::Text::new("LND");
                graphics::queue_text(ctx, &text, Point { x: -20.0, y: 55.0 }, Some(Color::GREEN));
            }

            if aircraft.on_ils.is_some() {
                let text = graphics::Text::new("LOC");
                graphics::queue_text(ctx, &text, Point { x: 20.0, y: 55.0 }, Some(Color::GREEN));
            }

            graphics::draw_queued_text(
                ctx,
                graphics::DrawParam::new().dest(aircraft.position),
                None,
                graphics::FilterMode::Linear,
            )?;
        }

        let selected_aircraft_text = graphics::Text::new(format!(
            "SELECTED: {}",
            self.aircraft[self.selected_aircraft as usize]
                .callsign
                .clone()
        ));
        graphics::queue_text(
            ctx,
            &selected_aircraft_text,
            Point { x: 0.0, y: 0.0 },
            Some(Color::WHITE),
        );
        graphics::draw_queued_text(
            ctx,
            graphics::DrawParam::new(),
            None,
            graphics::FilterMode::Linear,
        )?;

        graphics::present(ctx)
    }
}

fn main() {
    let (mut ctx, event_loop) = ContextBuilder::new("atc", "Antonis Kalou")
        .window_setup(ggez::conf::WindowSetup::default().title("ATC Simulator 2022"))
        .window_mode(ggez::conf::WindowMode::default().dimensions(1280.0, 960.0))
        .build()
        .expect("Could not create ggez context");

    let game = Game::new(&mut ctx);
    event::run(ctx, event_loop, game);
}
