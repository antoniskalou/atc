mod aircraft;
mod atc;
mod camera;
mod cli;
mod command;
mod geo;
mod geom;
mod math;
mod msfs_integration;
mod tts;
mod units;

use std::sync::Arc;
use std::sync::RwLock;

use crate::aircraft::*;
use crate::atc::*;
use crate::cli::*;
use crate::command::*;
use crate::geo::*;
use crate::geom::*;
use camera::Camera;
use ggez::{
    event::{self, EventHandler, KeyCode, MouseButton},
    graphics::{self, Color},
    timer, Context, ContextBuilder, GameResult,
};
use lazy_static::lazy_static;

const TTS_ENABLED: bool = false;

const AIRCRAFT_RADIUS: f32 = 4.0;
const AIRCRAFT_BOUNDING_RADIUS: f32 = AIRCRAFT_RADIUS * 5.0;

lazy_static! {
    // 34° 43' 5.08" N 32° 29' 6.26" E
    static ref PAPHOS_LATLON: LatLon = LatLon::from_dms(
        DMS::new(34, 43, 5.08, Cardinal::North),
        DMS::new(32, 29, 6.26, Cardinal::East)
    );
}

#[derive(Debug)]
struct Game {
    atc: Atc,
    cli: CliPrompt,
    msfs: msfs_integration::MSFS,
    airport: Airport,
    selected_aircraft: Option<usize>,
    aircraft: Arc<RwLock<Vec<Aircraft>>>,
    camera: Camera,
    screen_scale: f32,
}

impl Game {
    pub fn new(ctx: &mut Context) -> Self {
        let runway_29 = Runway {
            offset: Point { x: 0.0, y: 0.0 },
            heading: 285,
            length: 2700,
            width: 45,
            ils_max_altitude: 2000,
        };
        let aircraft = Arc::new(RwLock::new(vec![
            Aircraft {
                position: ggez::mint::Point2 { x: 0.0, y: 0.0 },
                callsign: Callsign {
                    name: "Cyprus Airways".into(),
                    code: "CYP".into(),
                    number: "2202".into(),
                },
                heading: HeadingParameter::new(90.0),
                altitude: AircraftParameter::new(6000.0),
                speed: AircraftParameter::new(240.0),
                status: AircraftStatus::Flight,
                cleared_to_land: false,
            },
            Aircraft {
                position: ggez::mint::Point2 {
                    x: 2000.0,
                    y: 3000.0,
                },
                callsign: Callsign {
                    name: "Fedex".into(),
                    code: "FDX".into(),
                    number: "261".into(),
                },
                heading: HeadingParameter::new(15.0),
                altitude: AircraftParameter::new(2000.0),
                speed: AircraftParameter::new(180.0),
                status: AircraftStatus::Flight,
                cleared_to_land: false,
            },
            Aircraft {
                position: ggez::mint::Point2 {
                    x: -2000.0,
                    y: -5000.0,
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
        ]));

        Self {
            atc: Atc::new(TTS_ENABLED),
            cli: CliPrompt::new(String::from("ATC>")),
            msfs: msfs_integration::MSFS::new(*PAPHOS_LATLON, aircraft.clone()),
            // msfs: msfs_integration::MSFS,
            airport: Airport {
                position: Point { x: 0.0, y: 0.0 },
                icao_code: "LCPH".into(),
                takeoff_runways: vec![runway_29.clone()],
                landing_runways: vec![runway_29.clone()],
            },
            selected_aircraft: None,
            camera: Camera::new(
                graphics::screen_coordinates(ctx).w,
                graphics::screen_coordinates(ctx).h,
            ),
            // 1m = 1/25 pixels
            screen_scale: 1. / 25.,
            aircraft,
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
                        let mut aircraft = self.aircraft.write().unwrap();
                        self.selected_aircraft.map(|sel| {
                            self.atc.command(&mut self.cli, &mut aircraft[sel], atc_cmd);
                        });
                    }
                    CliCommand::Comm(CommCommand::ListAircraft) => {
                        let aircraft = self.aircraft.read().unwrap();
                        for (idx, aircraft) in aircraft.iter().enumerate() {
                            self.cli
                                .output(format!("{}: {}", idx, aircraft.callsign.coded()));
                        }
                    }
                    CliCommand::Comm(CommCommand::ChangeAircraft(callsign)) => {
                        self.cli
                            .output(format!("Changing aircraft to {}", callsign));

                        let aircraft = self.aircraft.read().unwrap();
                        match aircraft_by_callsign(callsign.clone(), &aircraft) {
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

        let mut aircraft = self.aircraft.write().unwrap();
        for mut aircraft in &mut aircraft.iter_mut() {
            if !aircraft.is_grounded() {
                let speed_change = aircraft.speed.current(dt) * units::KT_TO_MS as f32 * dt;

                let heading = aircraft.heading.current(dt);
                let heading = heading_to_point(heading as i32);
                aircraft.position.x += speed_change * heading.x;
                aircraft.position.y += speed_change * heading.y;

                let _alt = aircraft.altitude.current(dt);
            }

            if aircraft.cleared_to_land() {
                // super inefficient
                for runway in &self.airport.landing_runways {
                    let origin = self.airport.origin(runway);
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

        let old_selection = self.selected_aircraft.and_then(|idx| aircraft.get(idx));
        let mut aircraft = aircraft.clone(); // need to clone for lifetimes

        // remove landed aircraft
        aircraft.retain(|a| !a.is_grounded());

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
        match keycode {
            KeyCode::LBracket => {
                self.selected_aircraft =
                    Some((self.selected_aircraft.unwrap_or(0) as i32 - 1).max(0) as usize);
            }
            KeyCode::RBracket => {
                self.selected_aircraft = Some(
                    (self.selected_aircraft.unwrap_or(0) + 1)
                        .min(self.aircraft.read().unwrap().len() - 1),
                );
            }
            KeyCode::W => {
                self.camera.move_by(Point { x: 0., y: 10. });
            }
            KeyCode::S => {
                self.camera.move_by(Point { x: 0., y: -10. });
            }
            KeyCode::A => {
                self.camera.move_by(Point { x: -10., y: 0. });
            }
            KeyCode::D => {
                self.camera.move_by(Point { x: 10., y: 0. });
            }
            KeyCode::F => {
                self.camera.move_to(Point { x: 0., y: 0. });
            }
            _ => {}
        }
    }

    fn mouse_button_down_event(&mut self, _ctx: &mut Context, button: MouseButton, x: f32, y: f32) {
        // aircraft selection
        if button == MouseButton::Left {
            let click_pos = Point { x, y };

            for (i, aircraft) in self.aircraft.read().unwrap().iter().enumerate() {
                if is_point_in_circle(click_pos, aircraft.position, AIRCRAFT_BOUNDING_RADIUS) {
                    self.selected_aircraft = Some(i);
                    break;
                }
            }
        }
    }

    fn mouse_wheel_event(&mut self, _ctx: &mut Context, _x: f32, y: f32) {
        // scale 1/50 pixels each scroll
        self.screen_scale = (self.screen_scale + 1. / 50. * y).max(0.02);
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, Color::BLACK);

        let aircraft = self.aircraft.read().unwrap();

        // scale line uses screen coords
        let scale_length = 1000. * self.screen_scale;
        let scale_points = [
            // uptick left
            Point { x: 0., y: -20.0 },
            Point { x: 0., y: -10.0 },
            // 1km scale
            Point {
                x: scale_length,
                y: -10.0,
            },
            // uptick right
            Point {
                x: scale_length,
                y: -20.0,
            },
        ];
        let scale_line = graphics::Mesh::new_line(ctx, &scale_points, 1., Color::GREEN)?;
        graphics::draw(
            ctx,
            &scale_line,
            (Point {
                x: 10.,
                y: self.camera.screen_size().x,
            },),
        )?;

        let scale_text = graphics::Text::new("1 KM");
        graphics::queue_text(
            ctx,
            &scale_text,
            Point {
                x: 10.0,
                y: self.camera.screen_size().y - 40.,
            },
            Some(Color::GREEN),
        );
        graphics::draw_queued_text(
            ctx,
            graphics::DrawParam::new(),
            None,
            graphics::FilterMode::Linear,
        )?;

        // airport
        let icao_text = graphics::Text::new(self.airport.icao_code.clone());
        graphics::queue_text(ctx, &icao_text, Point { x: 0.0, y: 0.0 }, Some(Color::BLUE));
        graphics::draw_queued_text(
            ctx,
            graphics::DrawParam::new().dest(self.camera.world_to_screen_coords(
                self.airport.position,
                self.screen_scale,
            )),
            None,
            graphics::FilterMode::Linear,
        )?;

        for runway in &self.airport.landing_runways {
            let origin = self.airport.origin(runway);
            let mesh = runway.as_mesh(ctx, origin, Color::RED, &self.camera, self.screen_scale)?;
            graphics::draw(ctx, &mesh, (Point { x: 0.0, y: 0.0 },))?;

            let ils = runway
                .ils(origin)
                .as_triangle()
                .iter()
                .map(|p| {
                    self.camera.world_to_screen_coords(
                        p.clone(),
                        self.screen_scale,
                    )
                })
                .collect::<Vec<Point>>();
            let mesh = graphics::Mesh::new_polygon(
                ctx,
                graphics::DrawMode::stroke(2.0),
                &ils,
                Color::BLUE,
            )?;
            graphics::draw(ctx, &mesh, (Point { x: 0.0, y: 0.0 },))?;
        }

        for aircraft in aircraft.iter() {
            let pos = self.camera.world_to_screen_coords(
                aircraft.position,
                self.screen_scale,
            );
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
                .and_then(|idx| aircraft.get(idx))
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
