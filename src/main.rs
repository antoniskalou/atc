use ggez::{
    event::{self, EventHandler, KeyCode, MouseButton},
    graphics::{self, Color},
    timer, Context, ContextBuilder, GameResult,
};

type Point = ggez::mint::Point2<f32>;

fn is_point_in_circle(point: Point, circle_pos: Point, circle_radius: f32) -> bool {
    (point.x - circle_pos.x).powi(2) + (point.y - circle_pos.y).powi(2) < circle_radius.powi(2)
}

fn rotate_point(point: Point, angle: f32) -> Point {
    Point {
        x: point.x * angle.cos() - point.y * angle.sin(),
        y: point.y * angle.cos() - point.x * angle.sin(),
    }
}

fn point_distance(p1: Point, p2: Point) -> f32 {
    ((p2.x - p1.x).powi(2) + (p2.y - p1.y).powi(2)).sqrt()
}

#[derive(Clone, Debug)]
struct PointRect {
    top_left: Point,
    top_right: Point,
    bottom_left: Point,
    bottom_right: Point,
}

impl PointRect {
    pub fn as_mesh(
        self,
        ctx: &mut Context,
        mode: graphics::DrawMode,
        color: Color,
    ) -> GameResult<graphics::Mesh> {
        // clockwise, order matters
        let points = [
            self.top_left,
            self.top_right,
            self.bottom_right,
            self.bottom_left,
        ];
        graphics::Mesh::new_polygon(ctx, mode, &points, color)
    }
}

/// Rotate a rectangle by an angle, in radians.
fn rotate_rect(rect: graphics::Rect, angle: f32) -> PointRect {
    let p1 = Point {
        x: rect.x,
        y: rect.y,
    };
    let p2 = Point {
        x: rect.x + rect.w,
        y: rect.y,
    };
    let p3 = Point {
        x: rect.x,
        y: rect.y + rect.h,
    };
    let p4 = Point {
        x: rect.x + rect.w,
        y: rect.y + rect.h,
    };

    println!("p1 (b): {:?}", p1);
    println!("p2 (b): {:?}", p2);
    println!("p3 (b): {:?}", p3);
    println!("p4 (b): {:?}", p4);

    let p1 = rotate_point(p1, angle);
    let p2 = rotate_point(p2, angle);
    let p3 = rotate_point(p3, angle);
    let p4 = rotate_point(p4, angle);

    println!("p1 (a): {:?}", p1);
    println!("p2 (a): {:?}", p2);
    println!("p3 (a): {:?}", p3);
    println!("p4 (a): {:?}", p4);

    // graphics::Rect::new(p1.x, p1.y, point_distance(p1, p2), point_distance(p1, p3))
    PointRect {
        top_left: p1,
        top_right: p2,
        bottom_left: p3,
        bottom_right: p4,
    }
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
}

const AIRCRAFT_RADIUS: f32 = 10.0;

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

#[derive(Clone, Debug)]
struct Runway {
    /// Offset from airport
    // offset: Point,
    heading: u32,
    /// length in meters
    length: u32,
    /// width in meters
    width: u32,
}

#[derive(Clone, Debug)]
struct Airport {
    // position: Point,
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
            heading: 290,
            length: 3000,
            width: 100,
        };

        Self {
            airports: vec![Airport {
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
                },
                Aircraft {
                    position: ggez::mint::Point2 { x: 400.0, y: 300.0 },
                    callsign: "TRA1112".into(),
                    heading: 180,
                    altitude: 12000,
                    speed: 220,
                    on_loc: false,
                    on_ils: false,
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
                if is_point_in_circle(click_pos, aircraft.position, AIRCRAFT_RADIUS) {
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
                graphics::DrawParam::new(),
                None,
                graphics::FilterMode::Linear,
            )?;

            for runway in &airport.landing_runways {
                let length_scale = 10.0;
                // let mesh = rotate_rect(
                //     graphics::Rect::new(
                //         200.0,
                //         200.0,
                //         runway.width as f32 / length_scale,
                //         runway.length as f32 / length_scale,
                //     ),
                //     (runway.heading as f32).to_radians(),
                // ).as_mesh(ctx, graphics::DrawMode::fill(), Color::BLUE)?;
                let runway_line = [
                    Point { x: 200.0, y: 200.0 },
                    Point { x: 300.0, y: 300.0},
                ];
                let mesh = graphics::Mesh::new_line(ctx, &runway_line, runway.width as f32 / length_scale, Color::BLUE)?;

                graphics::draw(ctx, &mesh, (Point { x: 0.0, y: 0.0 },))?;
            }
        }

        for aircraft in &self.aircraft {
            let circle = graphics::Mesh::new_circle(
                ctx,
                graphics::DrawMode::stroke(2.0),
                aircraft.position,
                AIRCRAFT_RADIUS,
                1.0,
                Color::GREEN,
            )?;

            graphics::draw(ctx, &circle, (Point { x: 0.0, y: 0.0 },))?;

            let callsign_text = graphics::Text::new(aircraft.callsign.clone());
            graphics::queue_text(
                ctx,
                &callsign_text,
                Point { x: -20.0, y: 10.0 },
                Some(Color::GREEN),
            );
            let heading_text = graphics::Text::new(format!("H{}", aircraft.heading));
            graphics::queue_text(
                ctx,
                &heading_text,
                Point { x: -20.0, y: 25.0 },
                Some(Color::GREEN),
            );
            let altitude_text = graphics::Text::new(format!("{}", aircraft.altitude));
            graphics::queue_text(
                ctx,
                &altitude_text,
                Point { x: 20.0, y: 25.0 },
                Some(Color::GREEN),
            );

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
