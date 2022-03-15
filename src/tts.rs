use std::process::{Command, Output};
use std::sync::mpsc::{self, Sender, Receiver};
use std::thread;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct TextToSpeech {
    thread: thread::JoinHandle<()>,
    queue: Sender<String>,
}

impl TextToSpeech {
    pub fn new() -> Self {
        let (tx, rx): (Sender<String>, Receiver<String>)  = mpsc::channel();

        let thread_rx = Arc::new(Mutex::new(rx));
        let thread = thread::spawn(move || {
            loop {
                let msg = thread_rx
                    .lock().unwrap()
                    .recv().unwrap();
                wsay(&msg).expect("wsay failed");
            }
        });
        

        Self { thread, queue: tx, }
    }

    pub fn say(&mut self, s: String) -> Result<(), mpsc::SendError<String>> {
        self.queue.send(s)
    }
}

fn wsay(s: &str) -> std::io::Result<Output> {
    Command::new("wsay.exe") 
        .arg(s)
        .output()
}
