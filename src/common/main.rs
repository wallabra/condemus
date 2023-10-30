use serde::{Serialize, Deserialize};
use crate::common::*;

#[derive(Clone, Serialize, Deserialize)]
pub struct TrackMetadata {
    pub name: String,
    pub init_tempo: f64,
    pub init_volume: f64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Track {
    pub pattern_refs: Vec<PatternRef>,
    pub metadata: TrackMetadata,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Project {
    pub patterns: Vec<Pattern>,
    pub samples: Vec<Sample>,
    pub instruments: Vec<Instrument>,
    pub tracks: Vec<Track>,
}
