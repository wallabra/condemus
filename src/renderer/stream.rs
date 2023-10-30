use super::resample::Resampler;
use std::ops::Range;
use std::rc::Arc;
use std::cell::RefCell;

pub struct Sink {
    pub out: Vec<f64>,
    pub rate: f64,
    pub resampler: Resampler,
}

impl Sink {
    pub fn len(&self) -> f64 {
        self.out.len() as f64 / self.rate
    }

    pub fn slice_all(&'a mut self) -> SinkSlice<'a> {
        SinkSlice {
            sink: self,
            start: 0.0,
            end: self.len()
        }
    }

    pub fn slice(&'a mut self, start: f64, end: f64) -> SinkSlice<'a> {
        SinkSlice {
            sink: self,
            start, end
        }
    }
}

impl Arc<RefCell<Sink>> {
    pub fn slice_all(&mut self) -> SinkSlice<'a> {
        RcSinkSlice {
            sink: self.clone(),
            start: 0.0,
            end: self.len()
        }
    }

    pub fn slice(&mut self, start: f64, end: f64) -> SinkSlice<'a> {
        SinkSlice {
            sink: self.clone(),
            start, end
        }
    }
}

pub struct SinkSlice<'a> {
    pub sink: &'a mut Sink,
    pub start: f64,
    pub end: f64,
}

impl<'a> SinkSlice<'a> {
    pub fn len_secs(&self) -> f64 {
        self.end - seld.start
    }

    pub fn len_samples(&self) -> usize {
        seld.len_secs() * self.sink.rate as usize
    }
}
