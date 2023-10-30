use serde::{Deserialize, Serialize};
use crate::common::*;

impl LoopDef {
    pub fn next_stop(&self, position: Position) -> Option<f64> {
        use LoopDef::*;
        match *self {
            None => Option::None,

            Forward(section) => Some(if position.reversing && position.at < section.from {
                0.0
            } else if position.reversing {
                section.from
            } else {
                section.to
            }),

            PingPong(section) => Some(if position.reversing && position.at < section.from {
                0.0
            } else if position.reversing {
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

            Forward(section) => Ok(if from.reversing && from.at < section.from {
                Position {
                    reversing: false,
                    at: 0.0,
                }
            } else {
                Position {
                    reversing: from.reversing,
                    at: if from.reversing {
                        section.to
                    } else {
                        section.from
                    },
                }
            }),

            PingPong(section) => Ok(if from.reversing && from.at < section.from {
                Position {
                    reversing: false,
                    at: 0.0,
                }
            } else {
                Position {
                    reversing: !from.reversing,
                    at: if from.reversing {
                        section.from
                    } else {
                        section.to
                    },
                }
            }),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct BasicMode {
    pub start: f64,
    pub loops: Vec<LoopDef>,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum SmoothingMode {
    None,
    Triangle,
    Linear(f64),
    SquareRoot(f64),
    Cosine(f64),
}

#[derive(Clone, Serialize, Deserialize)]
pub struct GranulatingMode {
    pub segment: LoopSection,
    pub interval: f64,
    pub gain: f64,
    pub smoothing: SmoothingMode,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum InstrumentMode {
    Basic(BasicMode),
    Granulating(GranulatingMode),
}

use super::renderer;
impl InstrumentMode {
    pub fn new_sampler<'a>(
        &self,
        data: std::sync::Arc<Project>,
        sample: usize,
    ) -> Box<dyn renderer::SamplerState + 'a>
    where
        'static: 'a,
    {
        match self {
            Self::Basic(def) => {
                Box::from(renderer::BasicSamplerState::new(data, sample, def.clone()))
            }
            Self::Granulating(def) => Box::from(renderer::GranulatingSamplerState::new(
                data,
                sample,
                def.clone(),
            )),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Instrument {
    pub sample: usize,
    pub volume: f64,
    pub pan: f64,
    pub base_pitch: f64,
    pub mode: InstrumentMode,
}

