use std::io::{self, BufWriter, Write};
use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;

const CLI_HEADER: &'static str = "
ATC Prompt
==========

Enter commands below.
";

#[derive(Debug)]
pub struct CliPrompt {
    thread: std::thread::JoinHandle<()>,
    input: Receiver<String>,
    output: Arc<(Mutex<BufWriter<io::Stdout>>, Condvar)>,
}

impl CliPrompt {
    pub fn new(prompt_text: String) -> Self {
        let (in_tx, in_rx) = mpsc::channel();
        let output = Arc::new((Mutex::new(BufWriter::new(io::stdout())), Condvar::new()));

        let thread = {
            let output = output.clone();
            thread::spawn(move || {
                println!("{}", CLI_HEADER);

                loop {
                    let line = prompt(&prompt_text);
                    in_tx.send(line.trim().to_string()).unwrap();

                    // output at least once
                    let (out, cvar) = &*output;
                    let out_lock = out.lock().unwrap();
                    let mut out_buf = cvar.wait(out_lock).unwrap();
                    out_buf.flush().unwrap();
                }
            })
        };

        Self {
            thread,
            output,
            input: in_rx,
        }
    }

    pub fn try_input(&self) -> Option<String> {
        self.input.try_recv().ok()
    }

    pub fn output<S: ToString>(&mut self, s: S) {
        let (out, _) = &*self.output;
        let mut buf = out.lock().unwrap();
        write!(buf, "{}\n", s.to_string()).unwrap()
    }

    /// unblock waiting for output, start receiving input again
    pub fn flush(&mut self) {
        let (_, cvar) = &*self.output;
        cvar.notify_all();
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
