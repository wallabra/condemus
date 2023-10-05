use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Sample {
    pub audio: Vec<f64>,
    pub baserate: f64,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct LoopSection {
    from: f64,
    to: f64,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub enum LoopDef {
    None,
    Forward(LoopSection),
    PingPong(LoopSection),
}

#[derive(Clone, Copy, Debug)]
pub struct Position {
    pub at: f64,
    pub reversing: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct Subseg {
    pub length: f64,
    pub from: Position,
}

impl LoopDef {
    pub fn next_stop(&self, position: Position) -> Option<f64> {
        use LoopDef::*;
        match *self {
            None => Option::None,

            Forward(section) => Some(if position.reversing {
                section.from
            } else {
                section.to
            }),

            PingPong(section) => Some(if position.reversing {
                section.from
            } else {
                section.to
            }),
        }
    }

    pub fn next_start(&self, from: Position) -> Result<Position, &str> {
        use LoopDef::*;
        match *self {
            None => Err("Cannot get the next start of a LoopDef::None!"),

            Forward(section) => Ok(Position {
                reversing: from.reversing,
                at: if from.reversing {
                    section.to
                } else {
                    section.from
                },
            }),

            PingPong(section) => Ok(Position {
                reversing: !from.reversing,
                at: if from.reversing {
                    section.from
                } else {
                    section.to
                },
            }),
        }
    }

    pub fn subsegs(&self, from: Position, after_secs: f64) -> Vec<Subseg> {
        let mut position = from;
        let mut subsegs: Vec<Subseg> = vec![];
        let mut remaining = after_secs;

        while remaining > 0.0 {
            let next_stop = self.next_stop(position);

            if next_stop.is_none() {
                subsegs.push(Subseg {
                    from: position,
                    length: remaining,
                });
                return subsegs;
            }

            let next_stop = next_stop.unwrap();

            let distance = (next_stop - position.at).abs();

            subsegs.push(Subseg {
                from: position,
                length: remaining.min(distance),
            });

            if remaining <= distance {
                return subsegs;
            }

            remaining -= distance;
            position = self.next_start(position).unwrap();
        }

        unreachable!()
    }
}

#[derive(Serialize, Deserialize)]
pub struct BasicMode {
    pub loops: Vec<LoopDef>,
}

#[derive(Serialize, Deserialize)]
pub enum SmoothingMode {
    None,
    Linear(f64),
    Cosine(f64),
}

#[derive(Serialize, Deserialize)]
pub struct GranulatingMode {
    pub segment: LoopSection,
    pub interval: f64,
    pub smoothing: SmoothingMode,
}

#[derive(Serialize, Deserialize)]
pub enum InstrumentMode {
    Basic(BasicMode),
    Granulating(GranulatingMode),
}

#[derive(Serialize, Deserialize)]
pub struct Instrument {
    pub sample: usize,
    pub volume: f64,
    pub pan: f64,
    pub mode: InstrumentMode,
}

#[derive(Serialize, Deserialize)]
pub struct Slide {
    pub length: f64,
    pub amount: f64,
}

#[derive(Serialize, Deserialize)]
pub struct Vibrato {
    depth: f64,
    period: f64,
}

#[derive(Serialize, Deserialize)]
pub enum Effect {
    Portamento(Slide),
    Vibrato(Vibrato),
}

#[derive(Serialize, Deserialize)]
pub struct NoteInstruction {
    pub pitch: Option<f64>,
    pub set_pan: Option<f64>,
    pub set_volume: Option<f64>,
    pub effects: Option<Vec<Effect>>,
}

#[derive(Serialize, Deserialize)]
pub enum Instruction {
    Note(NoteInstruction),
    Cut,
    Stop,
    NextLoop,
    Fade(f64),
    Pause,
}

#[derive(Serialize, Deserialize)]
pub enum CommandEffect {
    SetGlobalVolume(f64),
    SetTempo(f64),
    SlideTempo(Slide),
    SlideGlobalVolume(Slide),
}

#[derive(Serialize, Deserialize)]
pub struct Command {
    pub offset: f64,
    pub effect: CommandEffect,
}

#[derive(Serialize, Deserialize)]
pub struct Pattern {
    pub instructions: Vec<Instruction>,
    pub width: u16,
    pub commands: Vec<Command>,
}

#[derive(Serialize, Deserialize)]
pub struct PatternRef {
    pub position: f64,
    pub pattern: usize,
}

#[derive(Serialize, Deserialize)]
pub struct TrackMetadata {
    pub name: String,
    pub init_tempo: f64,
    pub init_volume: f64,
}

#[derive(Serialize, Deserialize)]
pub struct Track {
    pub pattern_refs: Vec<PatternRef>,
    pub metadata: TrackMetadata,
}

#[derive(Serialize, Deserialize)]
pub struct Project {
    pub samples: Vec<Sample>,
    pub instruments: Vec<Instrument>,
    pub tracks: Vec<Track>,
}
