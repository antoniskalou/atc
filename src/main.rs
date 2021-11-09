use ggez::{
    event::{self, EventHandler, KeyCode, MouseButton},
    graphics::{self, Color},
    timer, Context, ContextBuilder, GameResult,
};

type Point = ggez::mint::Point2<f32>;

fn is_point_in_circle(point: Point, circle_pos: Point, circle_radius: f32) -> bool {
    (point.x - circle_pos.x).powi(2) + (point.y - circle_pos.y).powi(2) < circle_radius.powi(2)
}

fn rotate_point(origin: Point, point: Point, angle: f32) -> Point {
    let cos = angle.cos();
    let sin = angle.sin();

    Point {
        x: (point.x - origin.x) * cos - (point.y - origin.y) * sin + origin.x,
        y: (point.y - origin.y) * cos - (point.x - origin.x) * sin + origin.y,
    }
}

fn rotate_points(origin: Point, points: &[Point], angle: f32) -> Vec<Point> {
    points
        .iter()
        .map(|p| rotate_point(origin, *p, angle))
        .collect()
}

fn heading_to_vector(heading: i32) -> Point {
    let heading = (heading as f32 - 90.0).to_radians();
    Point {
        x: heading.cos(),
        y: heading.sin(),
    }
}

fn main() {
    let (mut ctx, event_loop) = ContextBuilder::new("atc", "Antonis Kalou")
        .build()
        .expect("Could not create ggez context");

    let game = Game::new(&mut ctx);

    event::run(ctx, event_loop, game);
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
    on_loc: bool,
    on_ils: bool,
    cleared_to_land: bool,
}

const AIRCRAFT_RADIUS: f32 = 5.0;
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

#[derive(Clone, Debug)]
struct Game {
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
            airports: vec![Airport {
                position: Point { x: 200.0, y: 300.0 },
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
                    on_loc: false,
                    on_ils: false,
                    cleared_to_land: false,
                },
                Aircraft {
                    position: ggez::mint::Point2 { x: 400.0, y: 300.0 },
                    callsign: "TRA1112".into(),
                    heading: 180,
                    altitude: 12000,
                    speed: 220,
                    on_loc: false,
                    on_ils: false,
                    cleared_to_land: false,
                },
            ],
        }
    }
}

impl EventHandler<ggez::GameError> for Game {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        let dt = timer::delta(ctx);

        for mut aircraft in &mut self.aircraft {
            let speed_scale = 25.0;
            let speed_change = (aircraft.speed as f32 * dt.as_secs_f32()) / speed_scale;

            let heading = heading_to_vector(aircraft.heading);
            aircraft.position.x += speed_change * heading.x;
            aircraft.position.y += speed_change * heading.y;
        }

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
            aircraft.change_heading(aircraft.heading - 5);
        } else if keycode == KeyCode::D {
            aircraft.change_heading(aircraft.heading + 5);
        } else if keycode == KeyCode::S {
            aircraft.change_altitude(aircraft.altitude - 1000);
        } else if keycode == KeyCode::W {
            aircraft.change_altitude(aircraft.altitude + 1000);
        } else if keycode == KeyCode::F {
            aircraft.change_speed(aircraft.speed - 10);
        } else if keycode == KeyCode::R {
            aircraft.change_speed(aircraft.speed + 10);
        } else if keycode == KeyCode::L {
            aircraft.cleared_to_land = !aircraft.cleared_to_land;
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
                let origin = Point {
                    x: airport.position.x + runway.offset.x,
                    y: airport.position.y + runway.offset.y,
                };
                let mesh = runway.as_mesh(ctx, origin, Color::BLUE)?;
                graphics::draw(ctx, &mesh, (Point { x: 0.0, y: 0.0 },))?;

                let localizer_origin = Point {
                    // rotated runway line points
                    x: runway.as_line(origin)[1].x,
                    y: runway.as_line(origin)[1].y,
                };
                let localizer = [
                    localizer_origin,
                    // 3 degree variance
                    rotate_point(
                        localizer_origin,
                        Point {
                            x: localizer_origin.x,
                            y: localizer_origin.y + 200.0,
                        },
                        -3f32.to_radians(),
                    ),
                    rotate_point(
                        localizer_origin,
                        Point {
                            x: localizer_origin.x,
                            y: localizer_origin.y + 200.0,
                        },
                        3f32.to_radians(),
                    ),
                ];

                let localizer = rotate_points(
                    localizer_origin,
                    &localizer,
                    (runway.heading as f32).to_radians(),
                );

                let mesh = graphics::Mesh::new_polygon(
                    ctx,
                    graphics::DrawMode::stroke(2.0),
                    &localizer,
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

            if aircraft.cleared_to_land {
                let text = graphics::Text::new("LND");
                graphics::queue_text(ctx, &text, Point { x: 0.0, y: 55.0 }, Some(Color::GREEN));
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
