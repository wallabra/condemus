//! Track related code, including the Sequence.

use std::time::Duration;
use super::pattern::Pattern;
use id_arena::Id;

/// A reference to a Pattern.
pub struct PatternReference {
    start: Duration,
    pattern: Id<Pattern>
}

/// Holds metadata about a Track.
pub struct TrackMetadata {
    author: String,

}

/// A Track.
///
/// Holds its metadata and a pattern sequence.
pub struct Track {
    pub info: TrackMetadata,
    pub sequence: Vec<PatternReference>,
}
