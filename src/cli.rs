use std::io::Write;
use std::thread;
use std::sync::mpsc::{self, Sender, Receiver};

use std::sync::{Arc, atomic::{self, AtomicBool}};

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
    should_flush: Arc<AtomicBool>,
}

impl CliPrompt {
    pub fn new(prompt_text: String) -> Self {
        let (in_tx, in_rx): (Sender<String>, Receiver<String>) = mpsc::channel();
        let (out_tx, out_rx): (Sender<String>, Receiver<String>) = mpsc::channel();
        let should_flush = Arc::new(AtomicBool::new(false));

        let thread = {
            let flush = should_flush.clone();
            thread::spawn(move || {
                println!("{}", CLI_HEADER);

                loop {
                    let line = prompt(&prompt_text);
                    in_tx.send(line.trim().to_string()).unwrap();

                    // wait for output to be flushed
                    while !flush.load(atomic::Ordering::Acquire) { }
                    for output in out_rx.try_iter() {
                        println!("{}", output);
                    }
                    flush.store(false, atomic::Ordering::Release);
                }
            })
        };

        Self { 
            thread, 
            input: in_rx, 
            output: out_tx, 
            should_flush: should_flush,
        }
    }

    pub fn try_input(&self) -> Option<String> {
        self.input.try_recv().ok()
    }

    pub fn output<S: ToString>(&self, s: S) {
        // maybe todo: acquire lock when starting output, release when flush
        //             ==>  explicitly release output from main thread
        self.output.send(s.to_string()).unwrap()
    }

    /// unblock waiting for output, start receiving input again
    pub fn flush(&self) {
        self.should_flush.store(true, atomic::Ordering::Release)
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