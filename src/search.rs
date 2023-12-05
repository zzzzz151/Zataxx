use std::time::Instant;
use crate::types::*;
use crate::utils::*;
use crate::board::*;

pub const INFINITY: i16 = 32500;
pub const MAX_DEPTH: u8 = 255;

struct SearchData<'a> {
    board: &'a mut Board,
    start_time: Instant,
    turn_milliseconds: u32,
    best_move_root: Move
}

pub fn search(board: &mut Board, milliseconds: u32) -> Move
{
    let mut search_data = SearchData {
        board: board,
        start_time: Instant::now(),
        turn_milliseconds: milliseconds / 24,
        best_move_root: MOVE_NONE

    };

    // ID (Iterative deepening)
    for iteration_depth in 1..=MAX_DEPTH 
    {
        let best_move_before: Move = search_data.best_move_root;
        let iteration_score = negamax(&mut search_data, iteration_depth as i16, 0 as u16, -INFINITY, INFINITY);

        if is_time_up(search_data.start_time, search_data.turn_milliseconds) {
            search_data.best_move_root = best_move_before;
            break;
        }

        println!("info depth {} score {} time {} pv {}",
                 iteration_depth, 
                 iteration_score,
                 milliseconds_elapsed(search_data.start_time), 
                 move_to_str(search_data.best_move_root));
    }

    assert!(search_data.best_move_root != MOVE_NONE);
    search_data.best_move_root
}

fn negamax(search_data: &mut SearchData, depth: i16, ply: u16, mut alpha: i16, beta: i16) -> i16
{
    if is_time_up(search_data.start_time, search_data.turn_milliseconds) {
        return 0; 
    }

    let game_result: GameResult = search_data.board.get_game_result();
    if game_result == GameResult::Draw {
        return 0;
    }
    else if game_result == GameResult::WinRed {
        return if search_data.board.color == Color::Red {INFINITY} else {-INFINITY};
    }
    else if game_result == GameResult::WinBlue {
        return if search_data.board.color == Color::Blue {INFINITY} else {-INFINITY};
    }

    if depth <= 0 { return search_data.board.eval(); }

    let mut moves: [Move; 256] = [MOVE_NONE; 256];
    let num_moves = search_data.board.moves(&mut moves);

    let mut best_score: i16 = -INFINITY;
    let mut best_move: Move = MOVE_NONE;

    for i in 0..num_moves
    {
        let mov: Move = moves[i as usize];
        search_data.board.make_move(mov);
        let score = -negamax(search_data, depth - 1, ply + 1, -beta, -alpha);
        search_data.board.undo_move();

        if is_time_up(search_data.start_time, search_data.turn_milliseconds) {
            return 0; 
        }

        if score > best_score { best_score = score; }

        if score <= alpha { continue; } // Fail low

        alpha = score;
        best_move = mov;
        if ply == 0 { 
            search_data.best_move_root = mov; 
        }

        if score < beta { continue; }

        // Fail high / beta cutoff

        break; // Fail high / beta cutoff
    }

    best_score
}