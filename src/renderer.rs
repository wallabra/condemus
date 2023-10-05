use super::defs;
use super::defs::{Position, Subseg};
use super::stream::Sink;

pub trait SamplerState<'def> {
    fn render(&mut self, sink: &mut Sink<'_>, gain: f64);
}

pub struct BasicSamplerState<'def> {
    sample: &'def defs::Sample,
    def: &'def defs::BasicMode,
    position: defs::Position,
    curr_loop: usize,
}

impl<'def> BasicSamplerState<'def> {
    pub fn new(sample: &'def defs::Sample, def: &'def defs::BasicMode) -> Self {
        Self {
            sample,
            def,
            position: Position {
                at: def.start,
                reversing: false,
            },
            curr_loop: 0,
        }
    }

    fn this_loop(&self) -> Option<&'def defs::LoopDef> {
        self.def.loops.get(self.curr_loop)
    }

    fn render_subseg(&self, subseg: &defs::Subseg, sink: &mut Sink<'_>, offs: f64, gain: f64) {
        let start = subseg.from.at;
        let reversing = subseg.from.reversing;
        let length = subseg.length;

        let left = if reversing { start - length } else { start };
        let right = if reversing { start } else { start + length };

        let left_o = (left * sink.rate) as usize;
        let right_o = (right * sink.rate) as usize;

        let left_s = (left * self.sample.baserate) as usize;
        let right_s = (right * self.sample.baserate) as usize;

        sink.resampler.resample(
            &self.sample.audio[left_s..=right_s],
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

    fn set_position_after(&mut self, subseg: &defs::Subseg) {
        self.position = defs::Position {
            reversing: subseg.from.reversing,
            at: subseg.from.at + subseg.length * (if subseg.from.reversing { -1.0 } else { 1.0 }),
        };
    }
}

impl<'def> SamplerState<'def> for BasicSamplerState<'def> {
    fn render(&mut self, sink: &mut Sink<'_>, gain: f64) {
        let mut render_offs: f64 = 0.0;
        let length = sink.out.len() as f64 / sink.rate;

        let subsegs = self.subsegs(self.position, length);

        for subseg in &subsegs {
            self.render_subseg(&subseg, sink, render_offs, gain);
            render_offs += subseg.length;
        }

        self.set_position_after(subsegs.last().unwrap());
    }
}

struct GranuleState {
    pub at: f64,
    pub age: f64,
    pub volume: f64,
}

impl GranuleState {}

pub struct GranulatingSamplerState<'def> {
    sample: &'def defs::Sample,
    def: &'def defs::GranulatingMode,
    granules: Vec<GranuleState>,
    age: f64,
}

impl<'def> GranulatingSamplerState<'def> {
    pub fn new(sample: &'def defs::Sample, def: &'def defs::GranulatingMode) -> Self {
        Self {
            sample,
            def,
            granules: vec![],
            age: 0.0,
        }
    }

    fn render(&mut self, sink: &mut Sink<'_>, gain: f64) {
        for granule in &mut self.granules {
            // TODO
        }
    }
}

pub struct EffectState<'def> {
    def: &'def defs::EffectInstance,
    pos: f64,
}

impl<'def> EffectState<'def> {
    pub fn new(def: &'def defs::EffectInstance) -> Self {
        Self { def, pos: 0.0 }
    }

    pub fn apply(&mut self, channel: &mut ChannelState) {
        use defs::Effect::*;
        match self.def.effect {
            // vibrations use derivative of sine (d a*sin(b*x) / dx = a*b*cos(b*x))
            Vibrato(vibrato) => {
                channel.pitch += vibrato.speed
                    * vibrato.depth
                    * (self.pos * vibrato.speed * std::f64::consts::PI * 2.0).cos();
            }

            // vibrations use derivative of sine (d a*sin(b*x) / dx = a*b*cos(b*x))
            Tremolo(tremolo) => {
                channel.volume += tremolo.speed
                    * tremolo.depth
                    * (self.pos * tremolo.speed * std::f64::consts::PI * 2.0).cos();
            }

            // vibrations use derivative of sine (d a*sin(b*x) / dx = a*b*cos(b*x))
            Panbrello(panbrello) => {
                channel.panning += panbrello.speed
                    * panbrello.depth
                    * (self.pos * panbrello.speed * std::f64::consts::PI * 2.0).cos();
            }

            Portamento(portamento) => {}
        }
    }

    fn expired(&self) -> bool {
        self.pos >= self.def.length
    }

    pub fn advance(&mut self, delta_secs: f64) -> bool {
        self.pos += delta_secs;
        self.expired()
    }
}

pub struct ChannelState<'def> {
    volume: f64,
    panning: f64,
    pitch: f64,
    effects: Vec<EffectState<'def>>,
    sampler: Box<dyn SamplerState<'def>>,
    instrument: &'def defs::Instrument,
    sample: &'def defs::Sample,
}

impl<'def> ChannelState<'def> {
    pub fn new(sample: &'def defs::Sample, instrument: &'def defs::Instrument, pitch: f64) -> Self {
        Self {
            instrument,
            sample,
            pitch,
            sampler: instrument.mode.new_sampler(sample),
            effects: vec![],
            panning: 0.0,
            volume: 1.0,
        }
    }

    fn add_effects(&mut self, instruction: &'def defs::NoteInstruction) {
        for effect in &instruction.effects.unwrap_or(vec![]) {
            self.effects.push(EffectState::new(effect));
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
            .into_iter()
            .enumerate()
            .filter_map(|(i, x)| {
                if to_remove.contains(&i) {
                    None
                } else {
                    Some(x)
                }
            })
            .collect();
    }

    fn apply_effects(&mut self) {
        for effect in &mut self.effects {
            effect.apply(self);
        }
    }

    fn render(&mut self, left_sink: &mut Sink<'_>, right_sink: &mut Sink<'_>) {
        debug_assert!(left_sink.rate == right_sink.rate);

        let pitch_rate = 2.0_f64.powf((self.pitch - self.instrument.base_pitch) / 12.0);

        left_sink.rate *= pitch_rate;
        right_sink.rate *= pitch_rate;

        self.sampler
            .render(left_sink, self.volume * (1.0 - self.panning) / 2.0);

        self.sampler
            .render(right_sink, self.volume * (1.0 + self.panning) / 2.0);

        left_sink.rate /= pitch_rate;
        right_sink.rate /= pitch_rate;

        self.apply_effects();
    }
}

pub struct PatternState<'def> {
    pattern: &'def defs::Pattern,
    row: usize,
    row_speed: f64,
    channels: Vec<Option<ChannelState<'def>>>,
}

impl<'def> PatternState<'def> {
    pub fn new(pattern: &'def defs::Pattern) -> Self {
        let mut channels: Vec<Option<ChannelState<'def>>> = vec![];

        for _ in 0..pattern.width {
            channels.push(None);
        }

        Self {
            pattern,
            channels,
            row: 0,
            row_speed: pattern.row_speed,
        }
    }

    pub fn curr_instructions(&self) -> &[defs::Instruction] {
        &self.pattern.instructions
            [self.row * self.pattern.width as usize..(self.row + 1) * self.pattern.width as usize]
    }

    pub fn channels(&mut self) -> &mut Vec<Option<ChannelState<'def>>> {
        &mut self.channels
    }

    fn advance_one(&mut self) -> bool {
        self.row += 1;
        self.row >= self.pattern.instructions.len() / self.pattern.width as usize
    }

    pub fn render(&mut self, sink: &mut Sink<'_>) {
        // TODO
    }
}

pub struct RenderState<'def> {
    data: &'def defs::Project,
    curr_track: Option<&'def defs::Track>,
    pattern_states: Vec<PatternState<'def>>,
    position: f64,
}

impl<'def> RenderState<'def> {
    pub fn new(data: &'def defs::Project) -> Self {
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

                self.add_pattern_state(&self.data.patterns[pref.pattern]);
            }
        }
    }

    fn add_pattern_state(&mut self, pattern: &'def defs::Pattern) {
        self.pattern_states.push(PatternState::new(pattern));
    }

    pub fn set_track(&mut self, which: usize) {
        self.curr_track = Some(&self.data.tracks[which]);
        self.position = -1.0;
        self.initialize_pattern_states();
    }

    pub fn stop(&mut self) {
        self.curr_track = None;
    }

    fn playing_notes(&mut self) -> Vec<(&'def defs::Instruction, &mut Option<ChannelState>)> {
        let mut res: Vec<_> = vec![];

        for pattern_state in &self.pattern_states {
            let instructions = pattern_state.curr_instructions();

            for zipped in instructions.iter().zip(pattern_state.channels()) {
                res.push(zipped);
            }
        }

        res
    }

    pub fn render(&mut self, sink: &mut Sink<'_>) {
        sink.out.fill(0.0);

        if self.curr_track.is_none() {
            return;
        }

        let Some(curr_track) = self.curr_track;

        for (instruction, channel) in self.playing_notes() {}
    }
}
