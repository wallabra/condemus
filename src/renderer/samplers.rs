use crate::common;
use crate::common::*;
use dasp::Signal;

pub struct BasicSamplerState {
    data: Arc<Project>,
    def: BasicMode,
    sample: usize,
    position: Position,
    curr_loop: usize,
}

impl BasicSamplerState {
    fn get_sample(&self) -> &Sample {
        &self.data.samples[self.sample]
    }

    pub fn new(data: Arc<Project>, sample: usize, def: common::BasicMode) -> Self {
        let start = def.start;
        Self {
            data,
            sample,
            def,
            position: Position {
                at: start,
                reversing: false,
            },
            curr_loop: 0,
        }
    }

    pub fn next_loop(&mut self) {
        self.curr_loop += 1;
    }

    fn this_loop(&self) -> Option<&LoopDef> {
        self.def.loops.get(self.curr_loop)
    }

    fn render_subseg(&self, subseg: &Subseg, sink: dasp::, offs: f64, gain: f64) {
        let start = subseg.from.at + offs;
        let reversing = subseg.from.reversing;
        let length = subseg.length;

        let left = if reversing { start - length } else { start };
        let right = if reversing { start } else { start + length };

        let left_o = (left * sink.rate) as usize;
        let right_o = (right * sink.rate) as usize;

        let sample = self.get_sample();
        let left_s = (left * sample.baserate) as usize;
        let right_s = (right * sample.baserate) as usize;

        sink.resampler.resample(
            &sample.audio[left_s..=right_s],
            &mut sink.out[left_o..=right_o],
            gain,
        );
    }

    fn subsegs(&self, from: Position, after_secs: f64) -> Vec<Subseg> {
        let mut position = from;
        let mut subsegs: Vec<Subseg> = vec![];
        let mut remaining = after_secs;

        let this_loop = self.this_loop();

        if this_loop.is_none() {
            subsegs.push(Subseg {
                from: position,
                length: remaining,
            });
            return subsegs;
        }

        let this_loop = this_loop.unwrap();

        while remaining > 0.0 {
            let next_stop = this_loop.next_stop(position);

            if next_stop.is_none() {
                subsegs.push(Subseg {
                    from: position,
                    length: remaining,
                });
                return subsegs;
            }

            let next_stop = next_stop.unwrap();

            let distance = (next_stop - position.at).abs();

            subsegs.push(Subseg {
                from: position,
                length: remaining.min(distance),
            });

            if remaining <= distance {
                return subsegs;
            }

            remaining -= distance;
            position = this_loop.next_start(position).unwrap();
        }

        unreachable!()
    }

    fn set_position_after(&mut self, subseg: &Subseg) {
        self.position = Position {
            reversing: subseg.from.reversing,
            at: subseg.from.at + subseg.length * (if subseg.from.reversing { -1.0 } else { 1.0 }),
        };
    }
}

impl SamplerState for BasicSamplerState {
    fn render(&mut self, sink: AudioBufferSlice<'_>, gain: f64) {
        let mut render_offs: f64 = 0.0;
        let length = sink.len_samples();

        let subsegs = self.subsegs(self.position, length);

        for subseg in &subsegs {
            self.render_subseg(&subseg, sink, render_offs, gain);
            render_offs += subseg.length;
        }

        self.set_position_after(subsegs.last().unwrap());
    }

    fn next_loop(&mut self) -> bool {
        if self.curr_loop + 1 < self.def.loops.len() {
            self.curr_loop += 1;
            true
        } else {
            false
        }
    }
}

struct GranuleState {
    pub at: f64,
    pub age: f64,
    pub volume: f64,
}

impl GranuleState {
    pub fn new(&mut self, at: f64, age: f64, volume: f64) -> Self {
        Self { at, age, volume }
    }

    pub fn advance(&mut self, amount: f64) {
        self.age += amount;
        self.at += amount;
    }

    fn volume(&self, def: &'static GranulatingMode) -> f64 {
        let position = self.age / def.segment.len();
        let dist = f64::min(1.0 - position, position);

        use common::SmoothingMode::*;
        match &def.smoothing {
            None => 1.0,
            Triangle => dist * 2.0,
            Linear(width) => dist / width,
            SquareRoot(width) => (dist / width).sqrt(),
            Cosine(width) => ((1.0 - dist) * std::f64::consts::PI / width / 2.0).cos(),
        }
    }

    pub fn render(
        &mut self,
        sample: &Sample,
        def: &GranulatingMode,
        sink: AudioBufferSlice<'_>,
        gain: f64,
    ) {
        // WIP
    }
}

pub struct GranulatingSamplerState {
    data: Arc<Project>,
    sample: usize,
    def: GranulatingMode,
    granules: Vec<GranuleState>,
    age: f64,
}

impl GranulatingSamplerState {
    pub fn new(data: Arc<Project>, sample: usize, def: common::GranulatingMode) -> Self {
        Self {
            data,
            sample,
            def,
            granules: vec![],
            age: 0.0,
        }
    }
}

impl SamplerState for GranulatingSamplerState {
    fn next_loop(&mut self) -> bool {
        // no-op
        // WIP: find a use for NextLopo in granulating instruments
        //      (possibly cycling a list of GranulatingMode?)
        false
    }

    fn render(&mut self, sink: AudioBufferSlice<'_>, gain: f64) {
        for granule in &mut self.granules {
            granule.render(&self.data.samples[self.sample], &self.def, sink, gain);
        }
    }
}
