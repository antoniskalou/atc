use msfs::sim_connect::{
    SimConnect
};
use std::pin::Pin;

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

        Ok(Self { thread, })
    }
}