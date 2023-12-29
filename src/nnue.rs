use crate::types::*;
use crate::utils::*;

pub const HIDDEN_LAYER_SIZE: usize = 256;
pub const SCALE: i32 = 400;
pub const QA: i32 = 255;
pub const QB: i32 = 64;

#[repr(C)]
pub struct Net {
    feature_weights: [i16; 147 * HIDDEN_LAYER_SIZE],
    feature_biases: [i16; HIDDEN_LAYER_SIZE],
    output_weights: [i16; 2 * HIDDEN_LAYER_SIZE],
    output_bias: i16
}

static NET: Net = unsafe { std::mem::transmute(*include_bytes!("net2.nnue")) };

#[derive(Clone, Copy)]
#[repr(C, align(64))]
pub struct Accumulator {
    red: [i16; HIDDEN_LAYER_SIZE],
    blue: [i16; HIDDEN_LAYER_SIZE]
}

impl Accumulator {
    pub fn default() -> Self {
        Self {
            red: NET.feature_biases,
            blue: NET.feature_biases
        }
    }

    pub fn activate_blocker(&mut self, sq: Square)
    {
        for i in 0..HIDDEN_LAYER_SIZE {
            let idx: usize = i + sq as usize + 49 * 2 * HIDDEN_LAYER_SIZE;
            self.red[i] += NET.feature_weights[idx];
            self.blue[i] += NET.feature_weights[idx];
        }
    }

    pub fn update(&mut self, color: Color, sq: Square, activate: bool)
    {
        let red_idx: usize = color as usize * 49 + sq as usize;
        let blue_idx: usize = opp_color(color) as usize * 49 + sq as usize;

        for i in 0..HIDDEN_LAYER_SIZE {
            if activate {
                self.red[i] += NET.feature_weights[i + red_idx * HIDDEN_LAYER_SIZE];
                self.blue[i] += NET.feature_weights[i + blue_idx * HIDDEN_LAYER_SIZE];
            }
            else {
                self.red[i] -= NET.feature_weights[i + red_idx * HIDDEN_LAYER_SIZE];
                self.blue[i] -= NET.feature_weights[i + blue_idx * HIDDEN_LAYER_SIZE];
            }
        }
    }
}

pub fn evaluate(color: Color, accumulator: &Accumulator) -> i16
{
    let mut us: &[i16; HIDDEN_LAYER_SIZE] = &accumulator.red;
    let mut them: &[i16; HIDDEN_LAYER_SIZE] = &accumulator.blue;

    if color == Color::Blue {
        us = &accumulator.blue;
        them = &accumulator.red;
    }

    let mut sum: i32 = 0;
    for i in 0..HIDDEN_LAYER_SIZE {
        sum += screlu(us[i]) * NET.output_weights[i] as i32;
        sum += screlu(them[i]) * NET.output_weights[i + HIDDEN_LAYER_SIZE] as i32;
    }

    let eval = (sum / QA + NET.output_bias as i32) * SCALE / (QA * QB);
    clamp(eval, -MIN_WIN_SCORE as i32 + 1, MIN_WIN_SCORE as i32 - 1) as i16
}

#[inline]
fn screlu(x: i16) -> i32 {
    i32::from(x.clamp(0, QA as i16)).pow(2)
}
