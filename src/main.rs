mod aircraft;
mod atc;
mod cli;
mod command;
mod geom;
mod tts;

use crate::atc::*;
use crate::aircraft::*;
use crate::cli::*;
use crate::command::*;
use crate::geom::*;
use ggez::{
    event::{self, EventHandler, KeyCode, MouseButton},
    graphics::{self, Color},
    timer, Context, ContextBuilder, GameResult,
};

const TTS_ENABLED: bool = false;

const AIRCRAFT_RADIUS: f32 = 4.0;
const AIRCRAFT_BOUNDING_RADIUS: f32 = AIRCRAFT_RADIUS * 5.0;

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
            cli: CliPrompt::new(String::from("Atc>")),
            airports: vec![Airport {
                position: Point { x: 500.0, y: 550.0 },
                icao_code: "LCPH".into(),
                takeoff_runways: vec![runway_29.clone()],
                landing_runways: vec![runway_29.clone()],
            }],
            selected_aircraft: None,
            aircraft: vec![
                Aircraft {
                    position: ggez::mint::Point2 { x: 250.0, y: 200.0 },
                    callsign: Callsign {
                        name: "Cyprus Airways".into(),
                        code: "CYP".into(),
                        number: "2202".into(),
                    },
                    heading: 90,
                    altitude: 6000,
                    speed: 250,
                    on_ils: None,
                    status: AircraftStatus::Flight,
                },
                Aircraft {
                    position: ggez::mint::Point2 { x: 500.0, y: 400.0 },
                    callsign: Callsign {
                        name: "Transavia".into(),
                        code: "TRA".into(),
                        number: "1112".into(),
                    },
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

        if let Some(msg) = self.cli.try_input() {
            for cmd in CliCommand::from_string(msg) {
                match cmd {
                    CliCommand::Atc(atc_cmd) => {
                        self.selected_aircraft.map(|sel| {
                            self.atc.command(
                                &mut self.cli,
                                &mut self.aircraft[sel],
                                atc_cmd,
                            );
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
                let speed_scale = 25.0;
                let speed_change = (aircraft.speed as f32 * dt.as_secs_f32()) / speed_scale;

                let heading = heading_to_vector(aircraft.heading);
                aircraft.position.x += speed_change * heading.x;
                aircraft.position.y += speed_change * heading.y;
            }

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

        let aircraft = self.aircraft.clone(); // need to clone for lifetimes
        let old_selection = self.selected_aircraft
                .and_then(|idx| aircraft.get(idx));

        // remove landed aircraft
        self.aircraft.retain(|a| !a.is_grounded());

        // set to previously selected item, if exists
        self.selected_aircraft = old_selection
            .and_then(|old_selection| { 
                aircraft
                    .iter()
                    .position(|a| a == old_selection)
            });
            

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
            self.selected_aircraft = Some((self.selected_aircraft.unwrap_or(0) as i32 - 1).max(0) as usize);
        } else if keycode == KeyCode::RBracket {
            self.selected_aircraft = Some((self.selected_aircraft.unwrap_or(0) + 1).min(self.aircraft.len() - 1));
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

            let callsign_text = graphics::Text::new(aircraft.callsign.coded());
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
        .window_mode(ggez::conf::WindowMode::default().dimensions(1280.0, 960.0))
        .build()
        .expect("Could not create ggez context");

    let game = Game::new(&mut ctx);
    event::run(ctx, event_loop, game);
}
