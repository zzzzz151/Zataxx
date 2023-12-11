#![allow(dead_code)]

use std::mem;
use crate::types::*;
//use crate::utils::*;

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
    move_and_bound: u16, // first 12 bits move, last 2 bits bound
}

impl TTEntry
{
    pub fn adjusted_score(&self, ply: i16) -> i16
    {
        if self.score >= MIN_WIN_SCORE { 
            return self.score - ply;
        }
        if self.score <= -MIN_WIN_SCORE { 
            return self.score + ply;
        }
        self.score
    }

    pub fn get_move(&self) -> Move {
        [((self.move_and_bound & 0b1111_1100_0000_0000) >> 10) as Square, 
         ((self.move_and_bound & 0b0000_0011_1111_0000) >> 4) as Square]
    }

    pub fn get_bound(&self) -> Bound {
        let bound: Bound = unsafe { mem::transmute((self.move_and_bound & 0b11) as u8) };
        bound 
    }

    pub fn store_move_and_bound(&mut self, mov: Move, bound: Bound) {
        let from: u16 = mov[FROM] as u16;
        let to: u16 = mov[TO] as u16;
        self.move_and_bound = (from << 10) | (to << 4) | (bound as u16);
    }
}

pub const DEFAULT_TT_SIZE_MB: usize = 32;

pub struct TT {
    pub entries: Vec<TTEntry>,
}

impl TT
{
    pub fn new(size_mb: usize) -> Self
    {
        let num_entries: usize = (size_mb * 1024 * 1024 / std::mem::size_of::<TTEntry>()) as usize;
        println!("TT size: {} MB ({} entries)", size_mb, num_entries);
        
        TT 
        {
            entries: vec![TTEntry {
                zobrist_hash: 0,
                depth: 0,
                score: 0,
                move_and_bound: 0,
            }; num_entries]
        }
    }   

    pub fn reset(&mut self)
    {
        self.entries = vec![TTEntry {
                                zobrist_hash: 0,
                                depth: 0,
                                score: 0,
                                move_and_bound: 0,
                            }; self.entries.len()];
        println!("TT reset");
    }

}