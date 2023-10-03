//! Top level project code.

use super::instrument::Instrument;
use super::pattern::Pattern;
use super::sample::Sample;
use super::track::Track;
use id_arena::Arena;

/**
 * A Project.
 *
 * Holds content like instruments, samples, patterns, et cetera.
 *
 * A project is a collection of resources, and may actually harbor multiple Tracks.
 */
pub struct Project {
    instruments: Arena<Instrument>,
    samples: Arena<Sample>,
    patterns: Arena<Pattern>,
    tracks: Arena<Track>,
}

impl Project {
    /// Constructs a new, empty Project.
    pub fn new() -> Self {
        Self {
            instruments: Arena::new(),
            samples: Arena::new(),
            patterns: Arena::new(),
            tracks: Arena::new(),
        }
    }
}
