use std::io::Write;
use std::thread;
use std::sync::mpsc::{self, Sender, Receiver};

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
}

impl CliPrompt {
    pub fn new() -> Self {
        let (in_tx, in_rx): (Sender<String>, Receiver<String>) = mpsc::channel();
        let (out_tx, out_rx): (Sender<String>, Receiver<String>) = mpsc::channel();

        let thread = thread::spawn(move || {
            println!("{}", CLI_HEADER);

            loop {
                for output in out_rx.try_iter() {
                    println!("{}", output);
                }

                let line = prompt("ATC> ");
                in_tx.send(line).unwrap();
                // FIXME: output not received in time for other iteration
                thread::sleep(std::time::Duration::from_secs(1));
            }
        });

        Self { thread, input: in_rx, output: out_tx, }
    }

    pub fn try_input(&self) -> Option<String> {
        self.input.try_recv().ok()
    }

    pub fn output(&self, s: String) {
        self.output.send(s).unwrap()
    }
}

fn prompt(s: &str) -> String {
    print!("{}", s);
    std::io::stdout().flush().unwrap();

    let mut line = String::new();
    std::io::stdin()
        .read_line(&mut line)
        .expect("failed to read stdin line");
    line
}