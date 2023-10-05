/*!
Resampling modes.
 */

#[derive(Clone, Copy, Debug)]
pub enum Resampler {
    Nearest,
    Linear,
}

impl Resampler {
    fn interpolate(&self, from: &[f64], pos: f64) -> f64 {
        use Resampler::*;
        match *self {
            Nearest => from[pos.round() as usize],
            Linear => {
                let delta = pos - pos.floor();
                from[pos.floor() as usize] * (1.0 - delta) + from[pos.ceil() as usize] * delta
            }
        }
    }

    pub fn resample(&self, from: &[f64], to: &mut [f64]) {
        let rate_fw = to.len() as f64 / from.len() as f64;

        for (out_idx, out_ptr) in to.iter_mut().enumerate() {
            let pos = out_idx as f64 / rate_fw;
            *out_ptr = self.interpolate(from, pos);
        }
    }
}
