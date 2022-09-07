mod aircraft;
mod atc;
mod cli;
mod command;
mod geo;
mod geom;
mod math;
mod msfs;
mod tts;

use crate::aircraft::*;
use crate::atc::*;
use crate::cli::*;
use crate::command::*;
use crate::geo::*;
use crate::geom::*;
use dms_coordinates::{Cardinal, DMS};
use ggez::{
    event::{self, EventHandler, KeyCode, MouseButton},
    graphics::{self, Color},
    timer, Context, ContextBuilder, GameResult,
};

const TTS_ENABLED: bool = false;

const AIRCRAFT_RADIUS: f32 = 4.0;
const AIRCRAFT_BOUNDING_RADIUS: f32 = AIRCRAFT_RADIUS * 5.0;

// 34°43′06″N 32°29′06″E
// const PAPHOS_LATLONG: LatLon = LatLon {
//     lat: DMS {
//         degrees: 34,
//         minutes: 43,
//         seconds: 6.0,
//         cardinal: Some(Cardinal::North),
//     },
//     lon: DMS {
//         degrees: 32,
//         minutes: 29,
//         seconds: 6.0,
//         cardinal: Some(Cardinal::East),
//     },
// };

#[derive(Debug)]
struct Game {
    atc: Atc,
    cli: CliPrompt,
    airports: Vec<Airport>,
    selected_aircraft: Option<usize>,
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
            atc: Atc::new(TTS_ENABLED),
            cli: CliPrompt::new(String::from("ATC>")),
            airports: vec![Airport {
                position: Point { x: 0.0, y: 0.0 },
                icao_code: "LCPH".into(),
                takeoff_runways: vec![runway_29.clone()],
                landing_runways: vec![runway_29.clone()],
            }],
            selected_aircraft: None,
            aircraft: vec![
                Aircraft {
                    position: ggez::mint::Point2 {
                        x: -100.0,
                        y: -200.0,
                    },
                    callsign: Callsign {
                        name: "Cyprus Airways".into(),
                        code: "CYP".into(),
                        number: "2202".into(),
                    },
                    // heading: AircraftParameter::new(90.0),
                    // FIXME
                    heading: HeadingParameter::new(90.0),
                    altitude: AircraftParameter::new(6000.0),
                    speed: AircraftParameter::new(240.0),
                    status: AircraftStatus::Flight,
                    cleared_to_land: false,
                },
                Aircraft {
                    position: ggez::mint::Point2 { x: 100.0, y: 300.0 },
                    callsign: Callsign {
                        name: "Fedex".into(),
                        code: "FDX".into(),
                        number: "261".into(),
                    },
                    heading: HeadingParameter::new(15.0),
                    altitude: AircraftParameter::new(8000.0),
                    speed: AircraftParameter::new(230.0),
                    status: AircraftStatus::Flight,
                    cleared_to_land: false,
                },
                Aircraft {
                    position: ggez::mint::Point2 {
                        x: 200.0,
                        y: -400.0,
                    },
                    callsign: Callsign {
                        name: "Transavia".into(),
                        code: "TRA".into(),
                        number: "1112".into(),
                    },
                    heading: HeadingParameter::new(180.0),
                    altitude: AircraftParameter::new(4000.0),
                    speed: AircraftParameter::new(220.0),
                    status: AircraftStatus::Flight,
                    cleared_to_land: false,
                },
            ],
        }
    }
}

impl EventHandler<ggez::GameError> for Game {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        let dt = timer::delta(ctx).as_secs_f32();

        if let Some(msg) = self.cli.try_input() {
            for cmd in CliCommand::from_string(msg) {
                match cmd {
                    CliCommand::Atc(atc_cmd) => {
                        self.selected_aircraft.map(|sel| {
                            self.atc
                                .command(&mut self.cli, &mut self.aircraft[sel], atc_cmd);
                        });
                    }
                    CliCommand::Comm(CommCommand::ListAircraft) => {
                        for (idx, aircraft) in self.aircraft.iter().enumerate() {
                            self.cli
                                .output(format!("{}: {}", idx, aircraft.callsign.coded()));
                        }
                    }
                    CliCommand::Comm(CommCommand::ChangeAircraft(callsign)) => {
                        self.cli
                            .output(format!("Changing aircraft to {}", callsign));

                        match aircraft_by_callsign(callsign.clone(), &self.aircraft) {
                            Some((idx, aircraft)) => {
                                self.cli
                                    .output(format!("Now speaking to {}", aircraft.callsign));
                                self.selected_aircraft = Some(idx);
                            }
                            None => {
                                self.cli.output(format!(
                                    "Error: Aircraft with callsign {} doesn't exist",
                                    callsign
                                ));
                            }
                        }
                    }
                }
            }
        }
        self.cli.flush();

        for mut aircraft in &mut self.aircraft {
            if !aircraft.is_grounded() {
                // FIXME: use scale in meters
                let speed_scale = 50.0;
                let speed_change = (aircraft.speed.current(dt) * dt) / speed_scale;

                let heading = aircraft.heading.current(dt);
                let heading = heading_to_vector(heading as i32);
                aircraft.position.x += speed_change * heading.x;
                aircraft.position.y += speed_change * heading.y;

                let _alt = aircraft.altitude.current(dt);
            }

            if aircraft.cleared_to_land() {
                // super inefficient
                for airport in &self.airports {
                    for runway in &airport.landing_runways {
                        let origin = airport.origin(runway);
                        let ils = runway.ils(origin);

                        if runway.has_landed(origin, aircraft) {
                            aircraft.status = AircraftStatus::Landed;
                        } else if aircraft.is_localizer_captured(&ils) {
                            aircraft.status = AircraftStatus::Landing;
                            aircraft.change_heading(runway.heading as i32, None);

                            let expected_alt = ils.altitude(aircraft.position);
                            aircraft.change_altitude(expected_alt);
                        }
                    }
                }
            }
        }

        let aircraft = self.aircraft.clone(); // need to clone for lifetimes
        let old_selection = self.selected_aircraft.and_then(|idx| aircraft.get(idx));

        // remove landed aircraft
        self.aircraft.retain(|a| !a.is_grounded());

        // set to previously selected item, if exists
        self.selected_aircraft = old_selection
            .and_then(|old_selection| aircraft.iter().position(|a| a == old_selection));

        Ok(())
    }

    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        keycode: KeyCode,
        _keymod: event::KeyMods,
        _repeat: bool,
    ) {
        if keycode == KeyCode::LBracket {
            self.selected_aircraft =
                Some((self.selected_aircraft.unwrap_or(0) as i32 - 1).max(0) as usize);
        } else if keycode == KeyCode::RBracket {
            self.selected_aircraft =
                Some((self.selected_aircraft.unwrap_or(0) + 1).min(self.aircraft.len() - 1));
        }
    }

    fn mouse_button_down_event(&mut self, _ctx: &mut Context, button: MouseButton, x: f32, y: f32) {
        // aircraft selection
        if button == MouseButton::Left {
            let click_pos = Point { x, y };

            for (i, aircraft) in self.aircraft.iter().enumerate() {
                if is_point_in_circle(click_pos, aircraft.position, AIRCRAFT_BOUNDING_RADIUS) {
                    self.selected_aircraft = Some(i);
                    break;
                }
            }
        }
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, Color::BLACK);

        let screen_size = graphics::screen_coordinates(ctx);

        for airport in &self.airports {
            let icao_text = graphics::Text::new(airport.icao_code.clone());
            graphics::queue_text(ctx, &icao_text, Point { x: 0.0, y: 0.0 }, Some(Color::BLUE));
            graphics::draw_queued_text(
                ctx,
                graphics::DrawParam::new().dest(world_to_screen_coords(
                    screen_size.w,
                    screen_size.h,
                    airport.position,
                )),
                None,
                graphics::FilterMode::Linear,
            )?;

            for runway in &airport.landing_runways {
                let origin = airport.origin(runway);
                let mesh = runway.as_mesh(ctx, origin, Color::RED)?;
                graphics::draw(ctx, &mesh, (Point { x: 0.0, y: 0.0 },))?;

                let ils = runway
                    .ils(origin)
                    .as_triangle()
                    .iter()
                    .map(|p| world_to_screen_coords(screen_size.w, screen_size.h, p.clone()))
                    .collect::<Vec<Point>>();
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
            let pos = world_to_screen_coords(screen_size.w, screen_size.h, aircraft.position);
            let aircraft_rect = graphics::Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::fill(),
                graphics::Rect::new(
                    pos.x - AIRCRAFT_RADIUS,
                    pos.y - AIRCRAFT_RADIUS,
                    AIRCRAFT_RADIUS * 2.0,
                    AIRCRAFT_RADIUS * 2.0,
                ),
                Color::GREEN,
            )?;

            graphics::draw(ctx, &aircraft_rect, (Point { x: 0.0, y: 0.0 },))?;

            let bounding_circle = graphics::Mesh::new_circle(
                ctx,
                graphics::DrawMode::stroke(2.0),
                pos,
                AIRCRAFT_BOUNDING_RADIUS,
                1.0,
                Color::GREEN,
            )?;

            graphics::draw(ctx, &bounding_circle, (Point { x: 0.0, y: 0.0 },))?;

            let callsign_text = graphics::Text::new(aircraft.callsign.coded());
            graphics::queue_text(
                ctx,
                &callsign_text,
                Point { x: -20.0, y: 30.0 },
                Some(Color::GREEN),
            );
            let heading_text =
                graphics::Text::new(format!("H{}", aircraft.heading.current.round()));
            graphics::queue_text(
                ctx,
                &heading_text,
                Point { x: -20.0, y: 45.0 },
                Some(Color::GREEN),
            );
            let altitude_text = {
                // alt to FL
                let alt = (aircraft.altitude.current / 100.0).round();
                graphics::Text::new(format!("FL{}", alt as u32))
            };
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

            if aircraft.status == AircraftStatus::Landing {
                let text = graphics::Text::new("LOC");
                graphics::queue_text(ctx, &text, Point { x: 20.0, y: 55.0 }, Some(Color::GREEN));
            }

            graphics::draw_queued_text(
                ctx,
                graphics::DrawParam::new().dest(pos),
                None,
                graphics::FilterMode::Linear,
            )?;
        }

        let selected_aircraft_text = graphics::Text::new(format!(
            "SELECTED: {}",
            self.selected_aircraft
                .and_then(|idx| self.aircraft.get(idx))
                .map(|a| a.callsign.coded())
                .unwrap_or(String::from("None"))
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
        .window_mode(ggez::conf::WindowMode::default().dimensions(1600.0, 1200.0))
        .build()
        .expect("Could not create ggez context");

    let game = Game::new(&mut ctx);
    event::run(ctx, event_loop, game);
}
