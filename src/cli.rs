use std::io::Write;
use std::thread;
use std::sync::mpsc::{self, Sender, Receiver};
use std::sync::{Arc, Mutex, atomic::{self, AtomicBool}};
use std::io;

const CLI_HEADER: &'static str = "
ATC Prompt
==========

Enter commands below.
";

#[derive(Debug)]
pub struct CliPrompt {
    thread: std::thread::JoinHandle<()>,
    input: Receiver<String>,
    output: Sender<String>,
    flush_and_wait: Arc<AtomicBool>,
}

impl CliPrompt {
    pub fn new(prompt_text: String) -> Self {
        let (in_tx, in_rx) = mpsc::channel();
        let (out_tx, out_rx) = mpsc::channel();
        let flush_and_wait = Arc::new(AtomicBool::new(false));

        let thread = {
            let flush_and_wait = flush_and_wait.clone();
            thread::spawn(move || {
                println!("{}", CLI_HEADER);

                loop {
                    let line = prompt(&prompt_text);
                    in_tx.send(line.trim().to_string()).unwrap();

                    while !flush_and_wait.load(atomic::Ordering::Acquire) {
                        // wait for output to be flushed
                    }
                    
                    // output at least once
                    print!("{}\n", out_rx.recv().unwrap());
                    for out in out_rx.try_iter() {
                        print!("{}\n", out);
                    }

                    flush_and_wait.store(false, atomic::Ordering::Release);
                }
            })
        };

        Self { 
            thread, 
            flush_and_wait,
            input: in_rx, 
            output: out_tx, 
        }
    }

    pub fn try_input(&self) -> Option<String> {
        // maybe todo: block continuing after success
        self.input.try_recv().ok()
    }

    pub fn output<S: ToString>(&mut self, s: S) {
        self.output.send(s.to_string()).unwrap()
    }

    /// unblock waiting for output, start receiving input again
    pub fn flush(&mut self) {
        self.flush_and_wait.store(true, atomic::Ordering::Release)
    }
}

fn prompt(s: &str) -> String {
    print!("{} ", s);
    std::io::stdout().flush().unwrap();

    let mut line = String::new();
    std::io::stdin()
        .read_line(&mut line)
        .expect("failed to read stdin line");
    line
}