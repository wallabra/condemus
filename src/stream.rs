use super::resample::Resampler;

pub struct Sink<'a> {
    pub out: &'a mut [f64],
    pub rate: f64,
    pub resampler: Resampler,
}
