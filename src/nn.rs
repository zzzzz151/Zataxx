use crate::types::*;
use crate::utils::*;
use crate::board::*;

pub const HIDDEN_LAYER_SIZE: usize = 256;
pub const SCALE: i32 = 400;
pub const QA: i32 = 255;
pub const QB: i32 = 64;

#[repr(C)]
pub struct Net {
    feature_weights: [[i16; HIDDEN_LAYER_SIZE]; 2916],
    feature_biases: [i16; HIDDEN_LAYER_SIZE],
    output_weights: [i16; HIDDEN_LAYER_SIZE],
    output_bias: i16
}

static NET: Net = unsafe { std::mem::transmute(*include_bytes!("net5-256.bin")) };

pub fn evaluate(board: &Board) -> i32
{
    const PER_TUPLE: usize = 3usize.pow(4);
    const POWERS: [usize; 4] = [1, 3, 9, 27];
    const MASK: u64 = 0b0001_1000_0011;

    let mut hidden_layer: [i16; HIDDEN_LAYER_SIZE] = NET.feature_biases;
    let us = board.us();
    let them = board.them();

    for i in 0..6 {
        for j in 0..6 {
            let tuple = 6 * i + j;
            let mut input_neuron_idx = PER_TUPLE * tuple;
            let offset = 7 * i + j;
            let mut this_us = (us >> offset) & MASK;
            let mut this_them = (them >> offset) & MASK;

            while this_us > 0 {
                let mut sq: usize = pop_lsb(&mut this_us) as usize;
                if sq > 6 { sq -= 5; }
                input_neuron_idx += POWERS[sq];
            }

            while this_them > 0 {
                let mut sq: usize = pop_lsb(&mut this_them) as usize;
                if sq > 6 { sq -= 5; }
                input_neuron_idx += 2 * POWERS[sq];
            }

            for k in 0..HIDDEN_LAYER_SIZE {
                hidden_layer[k] += NET.feature_weights[input_neuron_idx][k];
            }
        }
    }

    let sum: i32;

    #[cfg(not(target_feature = "avx2"))]
    {
    sum = fallback::flatten(&hidden_layer, &NET.output_weights);
    }

    #[cfg(target_feature = "avx2")]
    unsafe {
    sum = avx2::flatten(&hidden_layer, &NET.output_weights);
    }

    let eval: i32 = (sum / QA + i32::from(NET.output_bias)) * SCALE / (QA * QB);
    eval.clamp(-MIN_WIN_SCORE + 1, MIN_WIN_SCORE - 1)
}


#[cfg(not(target_feature = "avx2"))]
mod fallback {
    use super::{HIDDEN_LAYER_SIZE, QA};

    #[inline]
    pub fn screlu(x: i16) -> i32 {
        i32::from(x.clamp(0, QA as i16)).pow(2)
    }

    #[inline]
    pub fn flatten(acc: &[i16; HIDDEN_LAYER_SIZE], weights: &[i16; HIDDEN_LAYER_SIZE]) -> i32 {
        let mut sum = 0;
        for (&x, &w) in acc.iter().zip(weights) {
            sum += screlu(x) * i32::from(w);
        }
        sum
    }
}

#[cfg(target_feature = "avx2")]
mod avx2 {
    use super::{HIDDEN_LAYER_SIZE, QA};
    use std::arch::x86_64::*;

    pub unsafe fn flatten(acc: &[i16; HIDDEN_LAYER_SIZE], weights: &[i16; HIDDEN_LAYER_SIZE]) -> i32 
    {
        use std::arch::x86_64::*;
        const CHUNK: usize = 16;
        let mut sum = _mm256_setzero_si256();
        let min = _mm256_setzero_si256();
        let max = _mm256_set1_epi16(QA as i16);

        for i in 0..(HIDDEN_LAYER_SIZE / CHUNK) {
            let mut v = load_i16s(acc, i * CHUNK);
            v = _mm256_min_epi16(_mm256_max_epi16(v, min), max);
            let w = load_i16s(weights, i * CHUNK);
            let product = _mm256_madd_epi16(v, _mm256_mullo_epi16(v, w));
            sum = _mm256_add_epi32(sum, product);
        }

        horizontal_sum_i32(sum)
    }

    #[inline]
    unsafe fn load_i16s(acc: &[i16; HIDDEN_LAYER_SIZE], start_idx: usize) -> __m256i {
        _mm256_load_si256(acc.as_ptr().add(start_idx).cast())
    }

    #[inline]
    unsafe fn horizontal_sum_i32(sum: __m256i) -> i32 {
        let upper_128 = _mm256_extracti128_si256::<1>(sum);
        let lower_128 = _mm256_castsi256_si128(sum);
        let sum_128 = _mm_add_epi32(upper_128, lower_128);
        let upper_64 = _mm_unpackhi_epi64(sum_128, sum_128);
        let sum_64 = _mm_add_epi32(upper_64, sum_128);
        let upper_32 = _mm_shuffle_epi32::<0b00_00_00_01>(sum_64);
        let sum_32 = _mm_add_epi32(upper_32, sum_64);
        _mm_cvtsi128_si32(sum_32)
    }
}

