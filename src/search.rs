#![allow(dead_code)]
#![allow(unused_variables)]

use rand::Rng;
use crate::types::*;
use crate::utils::*;
use crate::board::*;

pub fn search(board: &mut Board) -> Move 
{
    let mut moves: [Move; 256] = [MOVE_NONE; 256];
    let num_moves = board.moves(&mut moves);

    let mut rng = rand::thread_rng();
    let random_index = rng.gen_range(0..num_moves);
    moves[random_index as usize]
}