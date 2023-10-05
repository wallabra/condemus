use super::defs;
use super::stream::Sink;
use id_arena::{Arena, Id};

pub trait SamplerState<'def> {
    fn set_sample(&mut self, sample: &'def defs::Sample);
    fn render(&mut self, sink: &mut Sink<'_>);
}

pub struct BasicSamplerState<'def> {
    sample: &'def defs::Sample,
    def: &'def defs::BasicMode,
    position: defs::Position,
    curr_loop: usize,
}

impl<'def> BasicSamplerState<'def> {
    fn this_loop(&self) -> defs::LoopDef {
        self.def.loops[self.curr_loop]
    }

    fn render_subseg(&self, subseg: &defs::Subseg, sink: &mut Sink<'_>, offs: f64) {
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
        );
    }

    fn set_position_after(&mut self, subseg: &defs::Subseg) {
        self.position = defs::Position {
            reversing: subseg.from.reversing,
            at: subseg.from.at + subseg.length * (if subseg.from.reversing { -1.0 } else { 1.0 }),
        };
    }
}

impl<'def> SamplerState<'def> for BasicSamplerState<'def> {
    fn set_sample(&mut self, sample: &'def defs::Sample) {
        self.sample = sample;
    }

    fn render(&mut self, sink: &mut Sink<'_>) {
        let mut render_offs: f64 = 0.0;
        let length = sink.out.len() as f64 / sink.rate;

        let subsegs = self.this_loop().subsegs(self.position, length);

        for subseg in &subsegs {
            self.render_subseg(&subseg, sink, render_offs);
            render_offs += subseg.length;
        }

        self.set_position_after(subsegs.last().unwrap());
    }
}

pub struct EffectState<'def> {
    def: &'def defs::Effect,
}

pub struct ChannelState<'def> {
    playing: bool,
    volume: f64,
    panning: f64,
    effects: Vec<EffectState<'def>>,
    sampler: Option<Box<dyn SamplerState<'def>>>,
}

impl<'def> Default for ChannelState<'def> {
    fn default() -> Self {
        Self {
            playing: false,
            volume: 1.0,
            panning: 0.0,
            effects: vec![],
            sampler: None,
        }
    }
}

pub struct PatternState<'def> {
    pattern: &'def defs::Pattern,
    row: usize,
    channel_alloc: Vec<Id<ChannelState<'def>>>,
}

pub struct RenderState<'def> {
    pattern_states: Arena<PatternState<'def>>,
    channels: Arena<ChannelState<'def>>,
    channel_alloc: Vec<bool>,
}

impl<'def> Default for RenderState<'def> {
    fn default() -> Self {
        Self {
            pattern_states: Arena::new(),
            channels: Arena::new(),
            channel_alloc: vec![],
        }
    }
}

impl<'def> RenderState<'def> {
    fn alloc_channels(&mut self, up_to: usize) {
        while self.channels.len() < up_to {
            self.channels.alloc(ChannelState::default());
            self.channel_alloc.push(false);
        }
    }
}
