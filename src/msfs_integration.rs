use crate::{aircraft::{Aircraft, self}, geo::LatLon};
use msfs::sim_connect::{data_definition, SimConnect, InitPosition};
use std::{collections::HashMap, sync::{Arc, Mutex, mpsc::{Receiver, self}, RwLock}, hash::Hash};

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

#[derive(Copy, Clone, Debug)]
struct GenRequestID {
    counter: u32,
}

impl GenRequestID {
    fn new() -> Self {
        Self { counter: 0 }
    }

    fn unique(&mut self) -> RequestID {
        let id = self.counter;
        self.counter += 1;
        id
    }
}

#[derive(Debug)]
pub struct MSFS {
    thread: std::thread::JoinHandle<()>,
    // sim: Pin<Box<SimConnect>>,
    gen_request_id: GenRequestID,
    // assigned_objects: Receiver<(RequestID, ObjectID)>,
}

impl MSFS {
    pub fn new(origin: LatLon, aircraft: Arc<RwLock<Vec<Aircraft>>>) -> Self {
        let mut gen_request_id = GenRequestID::new();
        let (oid_tx, oid_rx) = mpsc::channel();
        let thread = std::thread::spawn(move || {
            let mut sim = SimConnect::open("ATC", |_sim, recv| match recv { 
                msfs::sim_connect::SimConnectRecv::AssignedObjectId(obj) => {
                    let request_id = obj.dwRequestID;
                    let object_id = obj.dwObjectID;
                    println!("Received rid: {}, oid: {}", request_id, object_id);
                    oid_tx.send((request_id, object_id)).unwrap();
                }
                _ => println!("SimConnect: {:?}", recv)
            }).expect("failed to start simconnect");
            
            let mut aircraft_requests = HashMap::new();
            for aircraft in aircraft.read().unwrap().iter() {
                let request_id = gen_request_id.unique();
                let init_pos = aircraft_to_init_pos(origin, aircraft.clone());
                sim.ai_create_non_atc_aircraft(
                    "PMDG 737-700 Transavia", 
                    &aircraft.callsign.coded(), 
                    init_pos,
                    request_id,
                ).unwrap();
                aircraft_requests.insert(request_id, aircraft.callsign.clone());
            }

            let mut aircraft_objects = HashMap::new();
            loop {
                sim.call_dispatch().expect("call dispatch");

                if let Ok((rid, oid)) = oid_rx.try_recv() {
                    sim.ai_release_control(oid, gen_request_id.unique()).unwrap();
                    
                    let aircraft = aircraft_requests.get(&rid).unwrap();
                    aircraft_objects.insert(oid, aircraft);
                }

                for (oid, callsign) in &aircraft_objects {
                    let aircraft = aircraft.read().unwrap();
                    let aircraft = aircraft.iter().find(|a| a.callsign == **callsign).unwrap();
                    let simdata = AIPlane {
                        altitude: aircraft.altitude.current as f64,
                        heading: aircraft.heading.current.to_radians() as f64,
                        airspeed: aircraft.speed.current as f64,
                    };
                    sim.set_data_on_sim_object(*oid, &simdata).unwrap();
                }

                std::thread::sleep(std::time::Duration::from_millis(UPDATE_FREQUENCY_MS));
            }
        });

        Self { 
            thread, 
            gen_request_id,
        }
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

fn flatten_option<T>(option: Option<Option<T>>) -> Option<T> {
    match option {
        None => None,
        Some(v) => v,
    }
}