#![allow(dead_code)]
#![allow(unused_variables)]

use lazy_static::lazy_static;
use std::sync::{Mutex, RwLock};
use std::time::Instant;
use crate::types::*;
use crate::utils::*;
use crate::board::*;

pub const INFINITY: i16 = 32500;
pub const MAX_DEPTH: u8 = 255;

lazy_static! {
    static ref START_TIME: Mutex<Instant> = Mutex::new(Instant::now());
    static ref TURN_MILLISECONDS: Mutex<u32> = Mutex::new(0);
}

pub fn search(board: &mut Board, milliseconds: u32) -> Move
{
    {
    let mut start_time = START_TIME.lock().unwrap();
    let mut turn_milliseconds = TURN_MILLISECONDS.lock().unwrap();
    *start_time = Instant::now();
    *turn_milliseconds = milliseconds / 24;
    }

    let mut best_move = MOVE_NONE;

    // ID (Iterative deepening)
    for iteration_depth in 1..=MAX_DEPTH 
    {
        let (iteration_score, iteration_move) = negamax(board, iteration_depth as i16, 0 as u16);

        if is_time_up() {
            break;
        }

        best_move = iteration_move;

        { 
        let start_time = START_TIME.lock().unwrap(); 
        println!("info depth {} time {} pv {}",
                 iteration_depth, milliseconds_elapsed(*start_time), move_to_str(best_move));
        }
    }

    assert!(best_move != MOVE_NONE);
    best_move
}

fn negamax(board: &mut Board, depth: i16, ply: u16) -> (i16, Move)
{
    if is_time_up() {
        return (0, MOVE_NONE);
    }

    let game_result: GameResult = board.get_game_result();
    if game_result == GameResult::Draw {
        return (0, MOVE_NONE);
    }
    else if game_result == GameResult::WinRed {
        return if board.color == Color::Red {(INFINITY, MOVE_NONE)} else {(-INFINITY, MOVE_NONE)};
    }
    else if game_result == GameResult::WinBlue {
        return if board.color == Color::Blue {(INFINITY, MOVE_NONE)} else {(-INFINITY, MOVE_NONE)};
    }

    if depth <= 0 {
        return (board.eval(), MOVE_NONE);
    }

    let mut moves: [Move; 256] = [MOVE_NONE; 256];
    let num_moves = board.moves(&mut moves);

    let mut best_score: i16 = -INFINITY;
    let mut best_move: Move = MOVE_NONE;

    for i in 0..num_moves
    {
        let mov: Move = moves[i as usize];
        board.make_move(mov);
        let (mut score, child_move) = negamax(board, depth - 1, ply + 1);
        score = -score;
        board.undo_move();

        if is_time_up() {
            return (0, MOVE_NONE);
        }

        if score >= best_score {
            best_score = score;
            best_move = mov;
        }
    }

    (best_score, best_move)
}

fn is_time_up() -> bool
{
    let start_time = START_TIME.lock().unwrap();
    let turn_milliseconds = TURN_MILLISECONDS.lock().unwrap();
    milliseconds_elapsed(*start_time) >= (*turn_milliseconds).into()

}