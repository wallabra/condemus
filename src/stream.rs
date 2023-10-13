use super::resample::Resampler;
use std::ops::Range;

pub struct Sink<'a> {
    pub out: &'a mut [f64],
    pub rate: f64,
    pub resampler: Resampler,
}

impl<'a> Sink<'a> {
    pub fn len(&self) -> f64 {
        self.out.len() as f64 * self.rate
    }

    pub fn slice<'b>(&'b mut self, index: Range<f64>) -> Self
    where
        'b: 'a,
    {
        let start = (index.start * self.rate) as usize;
        let end = (index.end * self.rate) as usize;

        Sink {
            out: &mut self.out[start..end],
            rate: self.rate,
            resampler: self.resampler,
        }
    }
}
