use super::resample::Resampler;
use std::ops::Range;
use std::sync::Arc;
use std::cell::RefCell;

pub struct AudioBuffer {
    pub out: Vec<f64>,
    pub rate: f64,
    pub resampler: Resampler,
}

impl AudioBuffer {
    pub fn len(&self) -> f64 {
        self.out.len() as f64 / self.rate
    }

    pub fn ref_buf<'a>(&'a mut self) -> AudioBufferSlice<'a> {
        AudioBufferSlice {
            sink: &mut self,
            start: 0.0,
            end: self.len()
        }
    }

    pub fn slice<'a>(&'a mut self, start: f64, end: f64) -> AudioBufferSlice<'a> {
        AudioBufferSlice {
            sink: &mut self,
            start, end
        }
    }
}

pub struct AudioBufferSlice<'a> {
    pub sink: &'a mut AudioBuffer,
    pub start: f64,
    pub end: f64,
}

impl<'a> AudioBufferSlice<'a> {
    pub fn len_secs(&self) -> f64 {
        self.end - self.start
    }

    pub fn len_samples(&self) -> usize {
        self.len_secs() * self.sink.rate as usize
    }
}

pub trait StereoSource {
    fn render<'a>(&mut self, left_sink: AudioBufferSlice<'a>, right_sink: AudioBufferSlice<'a>);
}
