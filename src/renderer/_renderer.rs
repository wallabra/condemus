use crate::common;
use crate::common::*;
use std::sync::Arc;

pub trait SamplerState {
    fn render(&mut self, sink: AudioBufferSlice<'_>, gain: f64);
    fn next_loop(&mut self) -> bool;
}

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

#[derive(Clone)]
pub struct EffectState {
    def: EffectInstance,
    pos: f64,
}

impl EffectState {
    pub fn new(def: EffectInstance) -> Self {
        Self { def, pos: 0.0 }
    }

    fn expired(&self) -> bool {
        self.pos >= self.def.length
    }

    pub fn advance(&mut self, delta_secs: f64) -> bool {
        self.pos += delta_secs;
        self.expired()
    }
}

pub struct ChannelState {
    volume: f64,
    panning: f64,
    pitch: f64,
    effects: Vec<EffectState>,
    sampler: Box<dyn SamplerState>,
    data: Arc<Project>,
    instrument: usize,
    sample: usize,
    paused: bool,
}

impl ChannelState {
    fn get_sample(&self) -> &Sample {
        &self.data.samples[self.sample]
    }

    fn get_instrument(&self) -> &Instrument {
        &self.data.instruments[self.instrument]
    }

    pub fn new(data: Arc<Project>, sample: usize, instrument: usize, pitch: f64) -> Self {
        let sampler = data.instruments[instrument]
            .mode
            .new_sampler(data.clone(), sample);

        Self {
            data,
            instrument,
            sample,
            pitch,
            sampler,
            effects: vec![],
            panning: 0.0,
            volume: 1.0,
            paused: false,
        }
    }

    pub fn from_instruction(data: Arc<Project>, ins: &common::NoteInstruction) -> Self {
        let sampler = data.instruments[ins.instrument]
            .mode
            .new_sampler(data.clone(), data.instruments[ins.instrument].sample);

        let sample = data.instruments[ins.instrument].sample;

        Self {
            data,
            instrument: ins.instrument,
            sample,
            pitch: ins.pitch,
            sampler,
            effects: ins
                .effects
                .iter()
                .cloned()
                .map(EffectState::new)
                .collect::<_>(),
            panning: ins.pan,
            volume: ins.volume,
            paused: false,
        }
    }

    pub fn stop(&mut self) {
        // WIP
    }

    pub fn fade(&mut self, amount_secs: f64) {
        // WIP
    }

    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }

    fn add_effects(&mut self, instruction: &'static NoteInstruction) {
        for effect in &instruction.effects {
            self.effects.push(EffectState::new(effect.clone()));
        }
    }

    fn advance_effects(&mut self, delta_secs: f64) {
        let mut to_remove: Vec<usize> = vec![];

        for (i, state) in self.effects.iter_mut().enumerate() {
            if state.advance(delta_secs) {
                to_remove.push(i);
            }
        }

        self.effects = self
            .effects
            .iter()
            .enumerate()
            .filter_map(|(i, x)| {
                if to_remove.contains(&i) {
                    None
                } else {
                    Some(x.clone())
                }
            })
            .collect();
    }

    fn apply_effects(&mut self) {
        for effect in &mut self.effects {
            use Effect::*;
            match &effect.def.effect {
                // vibrations use derivative of sine (d a*sin(b*x) / dx = a*b*cos(b*x))
                Vibrato(vibrato) => {
                    self.pitch += vibrato.speed
                        * vibrato.depth
                        * (effect.pos * vibrato.speed * std::f64::consts::PI * 2.0).cos();
                }

                // vibrations use derivative of sine (d a*sin(b*x) / dx = a*b*cos(b*x))
                Tremolo(tremolo) => {
                    self.volume += tremolo.speed
                        * tremolo.depth
                        * (effect.pos * tremolo.speed * std::f64::consts::PI * 2.0).cos();
                }

                // vibrations use derivative of sine (d a*sin(b*x) / dx = a*b*cos(b*x))
                Panbrello(panbrello) => {
                    self.panning += panbrello.speed
                        * panbrello.depth
                        * (effect.pos * panbrello.speed * std::f64::consts::PI * 2.0).cos();
                }

                Portamento(portamento) => {
                    // WIP: implement portamento
                }
            }
        }
    }

    fn render<'a>(
        &mut self,
        left_sink: AudioBufferSlice<'a>,
        right_sink: AudioBufferSlice<'a>,
        cap: f64,
    ) {
        if self.paused {
            return;
        }

        debug_assert!(left_sink.rate == right_sink.rate);

        let pitch_rate = 2.0_f64.powf((self.pitch - self.get_instrument().base_pitch) / 12.0);

        left_sink.rate *= pitch_rate;
        right_sink.rate *= pitch_rate;

        self.sampler
            .render(left_sink, 0, cap, self.volume * (1.0 - self.panning) / 2.0);

        self.sampler
            .render(right_sink, 0, cap, self.volume * (1.0 + self.panning) / 2.0);

        left_sink.rate /= pitch_rate;
        right_sink.rate /= pitch_rate;

        self.apply_effects();
    }

    pub fn next_loop(&mut self) -> bool {
        self.sampler.next_loop()
    }
}

pub struct PatternState {
    data: Arc<Project>,
    pattern: usize,
    row: usize,
    row_speed: f64,      // rows per second
    inner_position: f64, // varies from 0 to 1
    channels: Vec<Option<ChannelState>>,
}

impl PatternState {
    pub fn new(data: Arc<Project>, pattern: usize) -> Self {
        let mut channels: Vec<Option<ChannelState>> = vec![];

        for _ in 0..data.patterns[pattern].width {
            channels.push(None);
        }

        let row_speed = data.patterns[pattern].row_speed;

        Self {
            data,
            pattern,
            channels,
            row: 0,
            row_speed,
            inner_position: 0.0,
        }
    }

    fn get_pattern(&self) -> &Pattern {
        &self.data.patterns[self.pattern]
    }

    pub fn curr_instructions(&self) -> &[Instruction] {
        let pattern = self.get_pattern();
        let width = pattern.width as usize;

        &pattern.instructions[self.row * width..(self.row + 1) * width]
    }

    pub fn channels(&mut self) -> &mut Vec<Option<ChannelState>> {
        &mut self.channels
    }

    fn row_segment(&self) -> f64 {
        (1.0 - self.inner_position) / self.row_speed
    }

    fn subsegs(&self, secs: f64) -> Vec<f64> {
        // each f64 returned is the length of the subseg where 1.0 is a single row
        let mut remaining = secs;
        let mut pos = self.inner_position;
        let mut res: Vec<f64> = Vec::new();
        let mut row = self.row;
        let height = self.get_pattern().height as usize;

        loop {
            let to_next_row = (1.0 - pos) * self.row_speed;
            res.push(to_next_row.min(remaining));
            remaining -= to_next_row;
            if remaining <= 0.0 {
                break;
            }

            row += 1;
            if row >= height {
                break;
            }
            pos = 0.0;
        }

        res
    }

    fn render_subseg<'a>(
        &mut self,
        left_sink: AudioBufferSlice<'a>,
        right_sink: AudioBufferSlice<'a>,
        start: f64,
        subseg: f64,
    ) {
        let width = self.data.patterns[self.pattern].width as usize;
        let row_idx_start = width * self.row;
        let row =
            &self.data.patterns[self.pattern].instructions[row_idx_start..row_idx_start + width];
        let speed = self.row_speed;

        let cap = subseg / speed;

        for (channel, instruction) in self.channels.iter_mut().zip(row.iter()) {
            use Instruction::*;
            match instruction {
                None => {}
                Cut => {
                    *channel = Option::None;
                }
                Stop => {
                    if channel.is_none() {
                        return;
                    }
                    channel.as_mut().unwrap().stop();
                }
                NextLoop => {
                    if channel.is_none() {
                        return;
                    }
                    if !unsafe { channel.as_mut().unwrap_unchecked() }.next_loop() {
                        *channel = Option::None;
                    }
                }
                Fade(num) => {
                    if channel.is_none() {
                        return;
                    }
                    unsafe { channel.as_mut().unwrap_unchecked() }.fade(*num);
                }
                Pause => {
                    if channel.is_none() {
                        return;
                    }
                    unsafe { channel.as_mut().unwrap_unchecked() }.toggle_pause();
                }
                Note(note_ins) => {
                    *channel = Some(ChannelState::from_instruction(self.data.clone(), note_ins));
                }
            };

            if let Some(channel) = channel {
                channel.render(left_sink, right_sink, cap);
            }
        }
    }

    pub fn render<'a>(
        &mut self,
        left_sink: AudioBufferSlice<'a>,
        right_sink: AudioBufferSlice<'a>,
    ) -> bool {
        debug_assert!(self.row < self.get_pattern().height as usize);

        let secs = left_sink.out.len() as f64 / left_sink.rate;
        let subsegs = self.subsegs(secs);
        let mut start: f64 = 0.0;

        let sink_len = left_sink.len();

        if subsegs.is_empty() {
            return false;
        }

        let new_inner_position = unsafe { subsegs.iter().last().unwrap_unchecked() };
        let first_subseg = subsegs[0];
        self.render_subseg(&left_sink, &right_sink, start, first_subseg);
        start += first_subseg;

        if subsegs.len() > 1 {
            self.inner_position = 0.0;

            for seg in &subsegs[1..] {
                self.row += 1;
                self.render_subseg(&left_sink, &right_sink, start, *seg);
                start += seg / self.row_speed;
            }
        }

        true
    }
}

pub struct RenderState {
    data: Arc<Project>,
    curr_track: Option<&'static Track>,
    pattern_states: Vec<PatternState>,
    position: f64,
}

impl RenderState {
    pub fn new(data: Arc<Project>) -> Self {
        Self {
            data,
            curr_track: None,
            pattern_states: vec![],
            position: 0.0,
        }
    }

    fn initialize_pattern_states(&mut self) {
        if let Some(curr_track) = self.curr_track {
            for pref in &curr_track.pattern_refs {
                if pref.position > 0.0 {
                    continue;
                }

                self.add_pattern_state(pref.pattern);
            }
        }
    }

    fn add_pattern_state(&mut self, pattern: usize) {
        self.pattern_states
            .push(PatternState::new(self.data.clone(), pattern));
    }

    pub fn set_track(&mut self, which: usize) {
        self.curr_track = Some(&self.data.tracks[which]);
        self.position = -1.0;
        self.initialize_pattern_states();
    }

    pub fn stop(&mut self) {
        self.curr_track = None;
    }
}

impl StereoSource for RenderState {
    fn render<'a>(&mut self, left_sink: AudioBufferSlice<'a>, right_sink: AudioBufferSlice<'a>) {
        left_sink.out.fill(0.0);
        right_sink.out.fill(0.0);

        if self.curr_track.is_none() {
            return;
        }

        let curr_track = unsafe { self.curr_track.unwrap_unchecked() };

        for pattern_state in &mut self.pattern_states {
            pattern_state.render(left_sink, right_sink);
        }
    }
}
