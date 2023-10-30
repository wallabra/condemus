use serde::{Deserialize, Serialize};
use crate::common::*;

#[derive(Clone, Serialize, Deserialize)]
pub struct Sample {
    pub audio: Vec<f64>,
    pub baserate: f64,
}

