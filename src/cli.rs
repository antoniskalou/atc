use std::io::Write;
use std::thread;
use std::sync::mpsc::{self, Sender, Receiver};

const CLI_HEADER: &'static str = "
ATC Prompt
==========

Enter commands below.
";

#[derive(Clone, Debug)]
enum OutputCommand {
    Write(String),
    Flush,
}

#[derive(Debug)]
pub struct CliPrompt {
    thread: std::thread::JoinHandle<()>,
    input: Receiver<String>,
    output: Sender<OutputCommand>,
}

impl CliPrompt {
    pub fn new(prompt_text: String) -> Self {
        let (in_tx, in_rx): (Sender<String>, Receiver<String>) = mpsc::channel();
        let (out_tx, out_rx): (Sender<OutputCommand>, Receiver<OutputCommand>) = mpsc::channel();

        let thread = thread::spawn(move || {
            println!("{}", CLI_HEADER);

            loop {
                let line = prompt(&prompt_text);
                in_tx.send(line).unwrap();

                // FIXME: assumes output always comes after input
                // println!("{}", out_rx.recv().unwrap());

                for output in out_rx.iter() {
                    match output {
                        OutputCommand::Write(s) =>  print!("{}\n", s),
                        OutputCommand::Flush => { 
                            std::io::stdout().flush().unwrap();
                            break;
                        }
                    }
                }
            }
        });

        Self { 
            thread, 
            input: in_rx, 
            output: out_tx, 
        }
    }

    pub fn try_input(&self) -> Option<String> {
        self.input.try_recv().ok()
    }

    pub fn output<S: ToString>(&self, s: S) {
        self.output.send(OutputCommand::Write(s.to_string())).unwrap()
    }

    pub fn flush(&self) {
        self.output.send(OutputCommand::Flush).unwrap()
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