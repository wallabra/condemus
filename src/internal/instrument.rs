/*!
 * The instruments, which take audio data from samples and,
 * using extra information, loop points, and sampling modes, make instruments
 * out of it.
 */

/**
 * A sampler.
 *
 * Denotes a way to take audio from a sample.
 */
pub trait Sampler {
    // WIP: sampler
}

/// A Sampler which simply plays back a sample, or segment thereof, once.
pub struct BasicSampler {
    // WIP
}

impl Sampler for BasicSampler {
    // WIP
}

/// A Sample which plays a segment of a sample multiple overlapping times.
///
/// Much more powerful than the BasicSampler, but used to make new timbres out
/// of the sample.
pub struct GranulatingSampler {
    // WIP
}

impl Sampler for GranulatingSampler {
    // WIP
}
