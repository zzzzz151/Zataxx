use std::time::Instant;
//use crate::tables::*;
use crate::types::*;
use crate::utils::*;
use crate::board::*;
use crate::tt::*;

pub const MAX_DEPTH: u8 = 100;

pub struct SearchData {
    pub board: Board,
    pub start_time: Instant,
    pub milliseconds: u32,
    pub turn_milliseconds: u32,
    pub best_move_root: Move,
    pub tt: TT
}

pub fn search(search_data: &mut SearchData) -> Move
{
    search_data.best_move_root = MOVE_NONE;
    search_data.turn_milliseconds = search_data.milliseconds / 24;

    // ID (Iterative deepening)
    for iteration_depth in 1..=MAX_DEPTH 
    {
        let iteration_score = pvs(search_data, iteration_depth as i16, 0 as i16, -INFINITY, INFINITY);

        println!("info depth {} score {} time {} pv {}",
                 iteration_depth, 
                 iteration_score,
                 milliseconds_elapsed(search_data.start_time), 
                 move_to_str(search_data.best_move_root));
                 
        if is_time_up(search_data) { break; }
    }

    assert!(search_data.best_move_root != MOVE_NONE);
    search_data.best_move_root
}

fn pvs(search_data: &mut SearchData, mut depth: i16, ply: i16, mut alpha: i16, beta: i16) -> i16
{
    if is_time_up(search_data) {
        return 0; 
    }

    if ply > 0
    {
        let game_result: GameResult = search_data.board.get_game_result();
        if game_result == GameResult::Draw {
            return 0;
        }
        else if game_result == GameResult::WinRed {
            return if search_data.board.color == Color::Red {INFINITY - ply} else {-INFINITY + ply};
        }
        else if game_result == GameResult::WinBlue {
            return if search_data.board.color == Color::Blue {INFINITY - ply} else {-INFINITY + ply};
        }
    }

    if depth <= 0 { return search_data.board.eval(); }

    depth = clamp(depth, 0, MAX_DEPTH as i16);

    let tt_entry_index = search_data.board.zobrist_hash as usize % search_data.tt.entries.len();
    let tt_entry_probed: &TTEntry = &search_data.tt.entries[tt_entry_index];
    let tt_hit: bool = search_data.board.zobrist_hash == tt_entry_probed.zobrist_hash;

    if ply > 0 && tt_hit && tt_entry_probed.depth >= (depth as u8)
    && (tt_entry_probed.bound == Bound::Exact
    || (tt_entry_probed.bound == Bound::Lower && tt_entry_probed.score >= beta)
    || (tt_entry_probed.bound == Bound::Upper && tt_entry_probed.score <= alpha))
    {
        return tt_entry_probed.adjusted_score(ply);
    }

    let pv_node: bool = (beta as i32 - alpha as i32) > 1 || ply == 0;
    if !pv_node && depth <= 5
    {
        let eval = search_data.board.eval();
        if eval >= beta + depth * 100 {
            return eval;
        }
    }

    let mut moves: MovesArray = EMPTY_MOVES_ARRAY;
    let num_moves = search_data.board.moves(&mut moves);
    let tt_move = if tt_hit {tt_entry_probed.best_move} else {MOVE_NONE};

    // Score moves
    let mut moves_scores: [u8; 256] = [0; 256];
    if num_moves > 1 {
        for i in 0..num_moves { 
            let mov: Move = moves[i as usize];
            if mov == tt_move {
                moves_scores[i as usize] = 255;
            }
            else {
                moves_scores[i as usize] = (mov[TO] == mov[FROM]) as u8;
                moves_scores[i as usize] += search_data.board.num_adjacent_enemies(mov[TO]);
            }
        }
    }

    let mut best_score: i16 = -INFINITY;
    let mut best_move: Move = MOVE_NONE;
    let original_alpha = alpha;

    for i in 0..num_moves
    {
        let mov: Move = incremental_sort(&mut moves, num_moves, &mut moves_scores, i as usize);
        search_data.board.make_move(mov);

        // PVS (Principal variation search)
        let score = if i == 0 {
            -pvs(search_data, depth - 1, ply + 1, -beta, -alpha)
        } else {
            let null_window_score = -pvs(search_data, depth - 1, ply + 1, -alpha - 1, -alpha);
            if null_window_score > alpha && null_window_score < beta {
                -pvs(search_data, depth - 1, ply + 1, -beta, -alpha)
            } else {
                null_window_score
            }
        };

        search_data.board.undo_move();

        if is_time_up(search_data) {
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

    let tt_entry: &mut TTEntry = &mut search_data.tt.entries[tt_entry_index];
    tt_entry.zobrist_hash = search_data.board.zobrist_hash;
    tt_entry.depth = depth as u8;
    tt_entry.best_move = best_move;

    tt_entry.score = if best_score >= MIN_WIN_SCORE { best_score + ply }
                     else if best_score <= -MIN_WIN_SCORE { best_score - ply }
                     else { best_score };

    tt_entry.bound = if best_score <= original_alpha { Bound::Upper }
                     else if best_score >= beta { Bound::Lower }
                     else { Bound::Exact };

    best_score
}

fn is_time_up(search_data: &mut SearchData) -> bool {
    milliseconds_elapsed(search_data.start_time) >= search_data.turn_milliseconds
}