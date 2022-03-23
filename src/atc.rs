use crate::aircraft::Aircraft;
use crate::cli::CliPrompt;
use crate::command::AtcCommand;
use crate::tts;

#[derive(Debug)]
pub struct Atc {
    tts: Option<tts::TextToSpeech>,
}

impl Atc {
    pub fn new(enable_tts: bool) -> Self {
        Self {
            tts: if enable_tts {
                Some(tts::TextToSpeech::new())
            } else {
                None
            },
        }
    }

    pub fn command(&mut self, cli: &mut CliPrompt, aircraft: &mut Aircraft, cmd: AtcCommand) {
        // request
        cli.output(format!("==> {}, {}", aircraft.callsign, cmd.as_string()));
        if let Some(tts) = &mut self.tts {
            tts.say(format!(
                "{}, {}",
                aircraft.callsign.spoken(),
                cmd.as_string()
            ))
            .expect("failed to send tts message");
        }

        aircraft.command(AtcRequest(cmd));
    }
}

pub struct AtcRequest(pub AtcCommand);
pub struct AtcReply(pub AtcCommand);
