use crate::{aircraft::Aircraft, geo::LatLon};
use msfs::sim_connect::{data_definition, SimConnect, InitPosition};

// TODO: make private
#[data_definition]
#[derive(Copy, Clone, Debug)]
pub struct AIPlane {
    #[name = "PLANE ALTITUDE"]
    #[unit = "feet"]
    pub altitude: f64,
    #[name = "PLANE HEADING DEGREES MAGNETIC"]
    #[unit = "radians"]
    pub heading: f64,
    #[name = "AIRSPEED INDICATED"]
    #[unit = "knots"]
    pub airspeed: f64,
}

#[derive(Debug)]
pub struct MSFS {
    thread: std::thread::JoinHandle<()>,
    // sim: Pin<Box<SimConnect>>,
}

impl MSFS {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let thread = std::thread::spawn(move || {
            let mut sim = SimConnect::open("ATC", |_sim, recv| println!("SimConnect: {:?}", recv))
                .expect("failed to open simconnect connection");

            loop {
                sim.call_dispatch().expect("call dispatch");
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
        });

        Ok(Self { thread })
    }
}

fn aircraft_to_init_pos(origin: LatLon, aircraft: Aircraft) -> InitPosition {
    let latlon = LatLon::from_game_world(origin, aircraft.position);
    InitPosition {
        Airspeed: aircraft.speed.current as u32,
        Altitude: aircraft.altitude.current as f64,
        Bank: 0.0,
        Heading: aircraft.heading.current as f64, // degrees
        Latitude: latlon.latitude(),
        Longitude: latlon.longitude(),
        OnGround: 0,
        Pitch: 0.0,
    }
}