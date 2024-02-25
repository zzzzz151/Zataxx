#![allow(dead_code)]

use std::mem;
use crate::types::*;
//use crate::utils::*;
use crate::ataxx_move::*;

#[repr(u8)]
#[derive(Clone, Copy)]
#[derive(PartialEq)]
pub enum Bound {
    None = 0,
    Exact = 1,
    Lower = 2,
    Upper = 3
}

#[derive(Clone, Copy)]
#[repr(packed)]
pub struct TTEntry {
    pub zobrist_hash: u64,
    pub depth: u8,
    pub score: i16,
    move_and_bound: u16, // lowest 12 bits move, highest 2 bits bound
}

impl TTEntry
{
    pub fn default() -> Self {
        Self {
            zobrist_hash: 0,
            depth: 0,
            score: 0,
            move_and_bound: 0
        }
    }

    pub fn adjusted_score(&self, ply: u8) -> i16
    {
        if self.score >= MIN_WIN_SCORE as i16 { 
            return self.score - ply as i16;
        }
        if self.score <= -MIN_WIN_SCORE as i16 { 
            return self.score + ply as i16;
        }
        self.score
    }

    pub fn get_move(&self) -> AtaxxMove {
        AtaxxMove::from_u16(self.move_and_bound & 0b0000_1111_1111_1111)
    }

    pub fn get_bound(&self) -> Bound {
        let bound: Bound = unsafe { mem::transmute((self.move_and_bound >> 14) as u8) };
        bound 
    }

    pub fn set_move(&mut self, mov: AtaxxMove) {
        self.move_and_bound &= 0b1100_0000_0000_0000;
        self.move_and_bound |= mov.to_u16();
    }

    pub fn set_bound(&mut self, bound: Bound) {
        self.move_and_bound &= 0b0000_1111_1111_1111;
        self.move_and_bound |= (bound as u16) << 14;
    }

    pub fn set_move_and_bound(&mut self, mov: AtaxxMove, bound: Bound) {
        self.move_and_bound = mov.to_u16() | ((bound as u16) << 14);
    }
}
