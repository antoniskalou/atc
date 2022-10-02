use crate::{aircraft::Aircraft, geo::LatLon};
use lazy_static::lazy_static;
use msfs::sim_connect::{data_definition, InitPosition, SimConnect, SimConnectRecv};
use std::{
    collections::HashMap,
    sync::{mpsc, Arc, RwLock, atomic::{AtomicU32, Ordering}},
};

const UPDATE_FREQUENCY_MS: u64 = 100;

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

type RequestID = msfs::sys::SIMCONNECT_DATA_REQUEST_ID;
type ObjectID = msfs::sys::SIMCONNECT_OBJECT_ID;

#[derive(Debug)]
struct GenRequestID {
    counter: AtomicU32,
}

impl GenRequestID {
    fn new() -> Self {
        Self { counter: AtomicU32::new(0) }
    }

    fn unique(&self) -> RequestID {
        self.counter.fetch_add(1, Ordering::SeqCst)
    }
}

lazy_static! {
    static ref GEN_REQUEST_ID: GenRequestID = GenRequestID::new();
}

#[derive(Debug)]
pub struct MSFS;

impl MSFS {
    pub fn new(origin: LatLon, aircraft: Arc<RwLock<Vec<Aircraft>>>) -> Self {
        std::thread::spawn(move || {
            let (oid_tx, oid_rx) = mpsc::channel();
            let mut sim = SimConnect::open("ATC", |_sim, recv| match recv {
                SimConnectRecv::AssignedObjectId(obj) => {
                    let request_id = obj.dwRequestID;
                    let object_id = obj.dwObjectID;
                    println!("Received rid: {}, oid: {}", request_id, object_id);
                    oid_tx.send((request_id, object_id)).unwrap();
                }
                _ => println!("SimConnect: {:?}", recv),
            })
            .expect("failed to start simconnect");

            // TODO: handle adding new aircraft after init
            let mut aircraft_requests = HashMap::new();
            for aircraft in aircraft.read().unwrap().iter() {
                let request_id = GEN_REQUEST_ID.unique();
                let init_pos = aircraft_to_init_pos(origin, aircraft.clone());
                sim.ai_create_non_atc_aircraft(
                    "Just Flight 146-200QC TNT", // TODO: fetch model
                    &aircraft.callsign.coded(),
                    init_pos,
                    request_id,
                )
                .unwrap();
                aircraft_requests.insert(request_id, aircraft.callsign.clone());
            }

            let mut aircraft_objects = HashMap::new();
            loop {
                sim.call_dispatch().expect("call dispatch");

                if let Ok((rid, oid)) = oid_rx.try_recv() {
                    if let Some(aircraft) = aircraft_requests.get(&rid) {
                        sim.ai_release_control(oid, GEN_REQUEST_ID.unique())
                            .unwrap();
                        aircraft_objects.insert(oid, aircraft);
                    }
                }

                for (oid, callsign) in &aircraft_objects {
                    let aircraft = aircraft.read().unwrap();

                    match aircraft.iter().find(|a| a.callsign == **callsign) {
                        Some(simaircraft) => {
                            let simdata = AIPlane {
                                altitude: simaircraft.altitude.current as f64,
                                heading: simaircraft.heading.current.to_radians() as f64,
                                airspeed: simaircraft.speed.current as f64,
                            };
                            sim.set_data_on_sim_object(*oid, &simdata).unwrap();
                        }
                        // aircraft has been removed
                        // FIXME: check not removing other processes objects
                        None => {
                            sim.ai_remove_object(*oid, GEN_REQUEST_ID.unique()).unwrap();
                        }
                    }
                }

                std::thread::sleep(std::time::Duration::from_millis(UPDATE_FREQUENCY_MS));
            }
        });

        Self
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_gen_request_id() {
        let gen = GenRequestID::new();
        assert_eq!(0, gen.unique());
        assert_eq!(1, gen.unique());
    }
}