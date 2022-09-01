use crate::aircraft::{Callsign, TurnDirection};

#[derive(Clone, Debug)]
pub enum AtcCommand {
    ChangeHeading(i32),
    ChangeHeadingWithTurnDirection(i32, TurnDirection),
    ChangeAltitude(u32),
    ChangeSpeed(u32),
    ClearedToLand(bool),
}

impl AtcCommand {
    fn from_parts(parts: &Vec<&str>) -> Vec<AtcCommand> {
        let mut commands = Vec::new();
        let mut iter = parts.iter();
        while let Some(cmd_str) = iter.next() {
            let cmd = match *cmd_str {
                "LND" => Some(AtcCommand::ClearedToLand(true)),
                "HDG" => {
                    // TODO: error handling
                    let hdg = iter.next().unwrap();
                    Some(AtcCommand::ChangeHeading(hdg.parse::<i32>().unwrap()))
                }
                "TURNL" => {
                    let hdg = iter.next().unwrap();
                    Some(AtcCommand::ChangeHeadingWithTurnDirection(
                        hdg.parse::<i32>().unwrap(),
                        TurnDirection::Left,
                    ))
                }
                "TURNR" => {
                    let hdg = iter.next().unwrap();
                    Some(AtcCommand::ChangeHeadingWithTurnDirection(
                        hdg.parse::<i32>().unwrap(),
                        TurnDirection::Right,
                    ))
                }
                "ALT" => {
                    let alt = iter.next().unwrap();
                    Some(AtcCommand::ChangeAltitude(alt.parse::<u32>().unwrap()))
                }
                "SPD" => {
                    let spd = iter.next().unwrap();
                    Some(AtcCommand::ChangeSpeed(spd.parse::<u32>().unwrap()))
                }
                _ => None,
            };

            if let Some(cmd) = cmd {
                commands.push(cmd);
            }
        }
        commands
    }

    pub fn as_string(&self) -> String {
        use AtcCommand::*;
        match self {
            // for tts its better to print heading to 1 8 0 for example
            ChangeHeading(heading) => format!("heading to {}", heading),
            ChangeHeadingWithTurnDirection(heading, direction) => {
                format!("turn {} to {}", direction, heading)
            }
            ChangeAltitude(alt) => format!("altitude to {} feet", alt),
            ChangeSpeed(speed) => format!("speed to {}", speed),
            ClearedToLand(cleared) => String::from(if *cleared {
                "cleared to land"
            } else {
                "clearance to land cancelled"
            }),
        }
    }
}

#[derive(Clone, Debug)]
pub enum CommCommand {
    ChangeAircraft(Callsign),
    // ChangeAircrafyByIndex(usize),
    ListAircraft,
}

impl CommCommand {
    fn from_parts(parts: &Vec<&str>) -> Vec<CommCommand> {
        let mut commands = Vec::new();
        let mut iter = parts.iter();
        while let Some(cmd_str) = iter.next() {
            let cmd = match *cmd_str {
                "LIST" => {
                    // todo: add other subcommands
                    Some(CommCommand::ListAircraft)
                }
                "SEL" => {
                    let aircraft_code = iter.next().unwrap();
                    Callsign::from_string(aircraft_code.to_string())
                        .map(|callsign| CommCommand::ChangeAircraft(callsign))
                }
                other => Callsign::from_string(other.to_string())
                    .map(|callsign| CommCommand::ChangeAircraft(callsign)),
            };

            if let Some(cmd) = cmd {
                commands.push(cmd);
            }
        }
        commands
    }
}

#[derive(Clone, Debug)]
pub enum CliCommand {
    Atc(AtcCommand),
    Comm(CommCommand),
    // Options(OptionsCommand),
}

impl CliCommand {
    pub fn from_string(s: String) -> Vec<CliCommand> {
        let cmd_str = s.trim().to_uppercase();
        let command_parts: Vec<&str> = cmd_str.split(' ').collect();
        // atc commands have precedence
        let atc_cmd = AtcCommand::from_parts(&command_parts);
        if atc_cmd.is_empty() {
            CommCommand::from_parts(&command_parts)
                .iter()
                .map(|c| CliCommand::Comm(c.clone()))
                .collect()
        } else {
            atc_cmd.iter().map(|c| CliCommand::Atc(c.clone())).collect()
        }
    }
}
