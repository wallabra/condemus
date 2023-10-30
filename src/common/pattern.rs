use serde::{Deserialize, Serialize};
use crate::common::*;

#[derive(Clone, Serialize, Deserialize)]
pub struct Slide {
    pub length: f64,
    pub amount: f64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Vibration {
    pub speed: f64,
    pub depth: f64,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum Effect {
    Portamento(Slide),
    Vibrato(Vibration),
    Tremolo(Vibration),
    Panbrello(Vibration),
}

#[derive(Clone, Serialize, Deserialize)]
pub struct EffectInstance {
    pub length: f64,
    pub effect: Effect,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct NoteInstruction {
    pub instrument: usize,
    pub pitch: f64,
    pub pan: f64,
    pub volume: f64,
    pub effects: Vec<EffectInstance>,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum Instruction {
    None,
    Note(NoteInstruction),
    Cut,
    Stop,
    NextLoop,
    Fade(f64),
    Pause,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum CommandEffect {
    SetGlobalVolume(f64),
    SetTempo(f64),
    SlideTempo(Slide),
    SlideGlobalVolume(Slide),
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Command {
    pub offset: f64,
    pub effect: CommandEffect,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Pattern {
    pub instructions: Vec<Instruction>,
    pub width: u16,
    pub height: u16,
    pub commands: Vec<Command>,
    pub row_speed: f64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PatternRef {
    pub position: f64,
    pub pattern: usize,
}

