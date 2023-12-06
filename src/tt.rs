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
    pub best_move: Move,
    pub score: i16,
    pub bound: Bound,
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
                best_move: MOVE_NONE,
                score: 0,
                bound: Bound::None,
            }; num_entries]
        }
    }   

    pub fn reset(&mut self)
    {
        self.entries = vec![TTEntry {
                                zobrist_hash: 0,
                                depth: 0,
                                best_move: MOVE_NONE,
                                score: 0,
                                bound: Bound::None,
                            }; self.entries.len()];
        println!("TT reset");
    }

}