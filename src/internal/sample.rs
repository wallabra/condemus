use std::time::Duration;

/**
A sample.

Contains an usually small bit of audio, which can be used as a source by
instruments.
**/
pub struct Sample {
    /// The base sample-rate of the sample.
    ///
    /// Speeding up or slowing down the sample is done by multiplying this.
    baserate: f64,

    /// The audio buffer.
    buf: Vec<f64>,
}

impl Sample {
    /// Creates a Sample from a slice and a base sample-rate.
    ///
    /// ```
    /// use condemus::internal::sample::Sample;
    ///
    /// let my_sine_wave = (0..4096)
    ///     .map(|x| (x as f64 * std::f64::consts::PI * 4.0 / 4096.0).sin())
    ///     .collect::<Vec<_>>();
    /// let my_sample = Sample::from_slice(32768.0, &my_sine_wave);
    /// ```
    pub fn from_slice(baserate: f64, buf: &[f64]) -> Self {
        Self {
            baserate,
            buf: Vec::from(buf),
        }
    }

    /// Creates a zeroed out Sample that is len samples long.
    pub fn zeros_samples(baserate: f64, len: usize) -> Self {
        Self {
            baserate,
            buf: vec![0.0; len],
        }
    }

    /// Creates a zeroed out Sample using a Duration as a length.
    pub fn zeros_duration(baserate: f64, len: Duration) -> Self {
        Self {
            baserate,
            buf: vec![0.0; (len.as_secs_f64() / baserate) as usize],
        }
    }

    /// Retrieves the sample's length as a Duration, using the base sample-rate.
    pub fn length(&self) -> Duration {
        Duration::from_secs_f64(self.buf.len() as f64 / self.baserate)
    }

    /// Returns a &[f64] slice borrowing this Sample's buffer.
    pub fn to_slice(&self) -> &[f64] {
        &self.buf
    }

    /// Returns a &mut [f64] mutable slice borrowing this Sample's buffer.
    pub fn mut_slice(&mut self) -> &mut [f64] {
        &mut self.buf
    }

    /// Wraps this sample into a 'slot', aka an Option of Self.
    ///
    /// Unlike simply using Some(...), this automatically converts empty samples
    /// into Nones.
    pub fn as_slot(self) -> Option<Self> {
        if self.buf.is_empty() {
            None
        } else {
            Some(self)
        }
    }

    /// Converts an offset in seconds into the integer index of the sample at that point.
    ///
    /// ```
    /// use std::time::Duration;
    /// use condemus::internal::sample::Sample;
    ///
    /// let empty_second = Sample::zeros_duration(1000.0, Duration::from_secs(1));
    /// let index = empty_second.index_at(Duration::from_secs_f64(0.32));
    /// assert_eq!(index, 320)
    /// ```
    pub fn index_at(&self, offset: Duration) -> usize {
        (offset.as_secs_f64() * self.baserate).floor() as usize
    }
}

/// A sample slot.
///
/// Used to represent possibly null/empty samples in the sample collection.
pub type SampleSlot = Option<Sample>;
