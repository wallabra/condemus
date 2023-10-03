/*!
 * The instruments, which take audio data from samples and,
 * using extra information, loop points, and sampling modes, make instruments
 * out of it.
 */

use super::sample::Sample;
use id_arena::Id;

/// A designation for looping audio segments.
pub enum LoopType {
    None,
    Forward,
    PingPong,
}

/// A way in which granule volumes are gradually smoothed out or faded with time.
pub enum GranuleSmoothType {
    None,
    Linear(f64),
    SquareRoot(f64),
    Cosine(f64),
}

impl GranuleSmoothType {
    pub fn factor_at(&self, position: f64) -> f64 {
        let dist = f64::min(1.0 - position, position);

        use GranuleSmoothType::*;
        match *self {
            None => 1.0,
            Linear(width) => dist / width,
            SquareRoot(width) => (dist / width).sqrt(),
            Cosine(width) => ((1.0 - dist) * std::f64::consts::PI / width / 2.0).cos(),
        }
        .min(1.0)
    }
}

/**
 * A sampler.
 *
 * Extracts relevant segments of sample audio in configurable ways.
 */
pub trait Sampler {
    // WIP: sampler interface
}

/// A Sampler which simply plays back a sample, or segment thereof, once.
pub struct BasicSampler {
    /// Beginning offset of segment to play or loop, between 0 and 1.
    begin: f64,

    /// End offset of segment to play or loop, between 0 and 1.
    end: f64,

    /// How to loop the segment.
    loop_type: LoopType,
}

impl Sampler for BasicSampler {
    // WIP
}

/// A Sample which plays a segment of a sample multiple overlapping times.
///
/// Much more powerful than the BasicSampler, but used to make new timbres out
/// of the sample.
pub struct GranulatingSampler {
    /// Beginning offset of segment to granulate, between 0 and 1.
    begin: f64,

    /// End offset of segment to granulate, between 0 and 1.
    end: f64,

    /// Interval between each granule, relative to sample width, between 0 and 1.
    interval: f64,

    /// Granule smoothing/fading mode.
    smoothing: GranuleSmoothType,
}

impl Sampler for GranulatingSampler {
    // WIP
}

/// Instruent data.
pub struct Instrument {
    /// Index into a sample in the Project.
    sample: Id<Sample>,

    /// Sampler used in this instrument.
    sampler: Box<dyn Sampler>,

    /// Volume multiplier, between 0 and 1.
    volume: f64,
    // WIP: more instrument metadata
}
