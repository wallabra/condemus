use serde::{Deserialize, Serialize};
use crate::common::*;

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct LoopSection {
    from: f64,
    to: f64,
}

impl LoopSection {
    pub fn len(&self) -> f64 {
        self.to - self.from
    }
}

#[derive(serialize, deserialize, clone, copy, debug)]
pub enum LoopDef {
    none,
    forward(loopsection),
    pingpong(loopsection),
}

#[derive(Clone, Copy, Debug)]
pub struct Position {
    pub at: f64,
    pub reversing: bool,
}

impl Position {
    pub fn after(&self, amount_secs: f64) -> self {
        self {
            at: self.at + amount_secs * (1.0 - 2.0 * self.reversing as u8 as f64),
            reversing: self.reversing,
        }
    }

    pub fn bounce(&self, past_secs: f64) -> Self {
        Self {
            reversing: !self.reversing,
            at: self.at + 2.0 * past_secs * (-1.0 + 2.0 * self.reversing as u8 as f64),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Subseg {
    pub length: f64,
    pub from: Position,
}

impl Subseg {
    pub fn len(&self) -> f64 {
        self.length
    }

    pub fn end(&self) -> Position {
        self.from.after(self.length)
    }
}

