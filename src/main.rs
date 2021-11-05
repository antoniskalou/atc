use ggez::{
    Context, ContextBuilder, GameResult,
    graphics::{self, Color},
    event::{self, EventHandler, KeyCode},
    input, 
    timer,
};

type Point = ggez::mint::Point2<f32>;

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
    // position: Point,
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
    selected_aircraft: i32,
    aircraft: Vec<Aircraft>,
}

impl Game {
    pub fn new(_ctx: &mut Context) -> Self {
        let runway_29 = Runway { heading: 290, length: 3000, width: 100, };

        Self {
            airports: vec![
                Airport {
                    icao_code: "LCPH".into(),
                    takeoff_runways: vec![runway_29.clone()],
                    landing_runways: vec![runway_29.clone()],
                }
            ],
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
                }
            ]
        }
    }
}

impl EventHandler<ggez::GameError> for Game {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        let dt = timer::delta(ctx);

        let aircraft = &mut self.aircraft[self.selected_aircraft as usize];
        if input::keyboard::is_key_pressed(ctx, KeyCode::A) {
            aircraft.change_heading(aircraft.heading - 5);
        } else if input::keyboard::is_key_pressed(ctx, KeyCode::D) {
            aircraft.change_heading(aircraft.heading + 5);
        } else if input::keyboard::is_key_pressed(ctx, KeyCode::S) {
            aircraft.change_altitude(aircraft.altitude - 1000);
        } else if input::keyboard::is_key_pressed(ctx, KeyCode::W) {
            aircraft.change_altitude(aircraft.altitude + 1000);
        } else if input::keyboard::is_key_pressed(ctx, KeyCode::F) {
            aircraft.change_speed(aircraft.speed - 10);
        } else if input::keyboard::is_key_pressed(ctx, KeyCode::R) {
            aircraft.change_speed(aircraft.speed + 10);
        } else if input::keyboard::is_key_pressed(ctx, KeyCode::RBracket) {
            self.selected_aircraft = (self.selected_aircraft + 1).min(self.aircraft.len() as i32 - 1);
            println!("{}", self.selected_aircraft);
        } else if input::keyboard::is_key_pressed(ctx, KeyCode::LBracket) {
            self.selected_aircraft = (self.selected_aircraft - 1).max(0);
            println!("{}", self.selected_aircraft);
        }

        for mut aircraft in &mut self.aircraft {
            let speed_scale = 25.0;
            let speed_change = (aircraft.speed as f32 * dt.as_secs_f32()) / speed_scale;
            // TODO: bearing to vector
            let heading = (aircraft.heading as f32 - 90.0).to_radians();
            aircraft.position.x += speed_change * heading.cos();
            aircraft.position.y += speed_change * heading.sin();
        }

        Ok(())
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
                graphics::FilterMode::Linear
            )?;

            for runway in &airport.landing_runways {
                let length_scale = 10.0;
                let rectangle = graphics::Mesh::new_rectangle(
                    ctx, 
                    graphics::DrawMode::fill(), 
                    graphics::Rect::new(
                        200.0, 200.0, runway.width as f32 / length_scale, runway.length as f32 / length_scale,
                    ),
                    Color::BLUE,
                )?;

                graphics::draw(ctx, &rectangle, (Point { x: 0.0, y: 0.0 },))?;
            }
        }

        for aircraft in &self.aircraft {
            let circle = graphics::Mesh::new_circle(
                ctx,
                graphics::DrawMode::stroke(2.0),
                aircraft.position, 
                10.0, 
                1.0, 
                Color::GREEN,
            )?;

            graphics::draw(ctx, &circle, (Point { x: 0.0, y: 0.0 },))?;

            let callsign_text = graphics::Text::new(aircraft.callsign.clone());
            graphics::queue_text(ctx, &callsign_text, Point { x: -20.0, y: 10.0, }, Some(Color::GREEN));
            let heading_text = graphics::Text::new(format!("H{}", aircraft.heading));
            graphics::queue_text(ctx, &heading_text, Point { x: -20.0, y: 25.0, }, Some(Color::GREEN));
            let altitude_text = graphics::Text::new(format!("{}", aircraft.altitude));
            graphics::queue_text(ctx, &altitude_text, Point { x: 20.0, y: 25.0, }, Some(Color::GREEN));

            graphics::draw_queued_text(
                ctx, 
                graphics::DrawParam::new().dest(aircraft.position),
                None,
                graphics::FilterMode::Linear,
            )?;
        }

        let selected_aircraft_text = graphics::Text::new(
            format!("SELECTED: {}", self.aircraft[self.selected_aircraft as usize].callsign.clone())
        );
        graphics::queue_text(ctx, &selected_aircraft_text, Point { x: 0.0, y: 0.0 }, Some(Color::WHITE));
        graphics::draw_queued_text(
            ctx, 
            graphics::DrawParam::new(), 
            None, 
            graphics::FilterMode::Linear
        )?;

        graphics::present(ctx)
    }
}
