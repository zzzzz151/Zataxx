use std::time::Instant;
use crate::tables::*;
use crate::types::*;
use crate::utils::*;
use crate::board::*;
use crate::tt::*;
use crate::nnue::*;

pub struct SearchData {
    pub board: Board,
    pub max_depth: u8,
    pub best_move_root: Move,
    pub start_time: Instant,
    pub milliseconds: u64,
    pub time_is_up: bool,
    pub nodes: u64,
    pub root_move_nodes: [[u64; 49]; 49],
    pub soft_nodes: u64,
    pub hard_nodes: u64,
    pub tt: TT,
    pub lmr_table: [[u8; 256]; 256]
}

impl SearchData
{
    pub fn new(board: Board, max_depth: u8, milliseconds: u64, soft_nodes: u64, hard_nodes: u64) -> Self {
        Self {
            board: board,
            max_depth: max_depth,
            best_move_root: MOVE_NONE,
            start_time: Instant::now(),
            milliseconds: milliseconds,
            time_is_up: false,
            nodes: 0,
            root_move_nodes: [[0; 49]; 49],
            soft_nodes: soft_nodes,
            hard_nodes: hard_nodes,
            tt: TT::new(DEFAULT_TT_SIZE_MB),
            lmr_table: get_lmr_table()
        }
    }

    pub fn is_hard_time_up(&mut self) -> bool 
    {
        if self.time_is_up || self.nodes >= self.hard_nodes { 
            return true; 
        }
        if (self.nodes % 1024) != 0 {
            return false;
        }
        self.time_is_up = milliseconds_elapsed(self.start_time) >= (self.milliseconds / 2);
        self.time_is_up
    }

    pub fn is_soft_time_up(&self) -> bool 
    {
        if self.nodes >= self.soft_nodes {
            return true;
        }

        let move_nodes_fraction: f64 = if self.best_move_root == MOVE_PASS { 
                                           1.0 
                                       } else {
                                           let best_move_nodes = self.root_move_nodes
                                                                 [self.best_move_root[FROM] as usize]
                                                                 [self.best_move_root[TO] as usize];
                                           best_move_nodes as f64 / self.nodes.max(1) as f64
                                       };

        let soft_time_scale = (0.5 + 1.0 - move_nodes_fraction) * 1.5;

        milliseconds_elapsed(self.start_time) 
        >= ((self.milliseconds as f64 * 0.05 * soft_time_scale) as u64)
    }
}

pub fn search(search_data: &mut SearchData, print_info: bool) -> (Move, i16)
{
    search_data.best_move_root = MOVE_NONE;
    search_data.time_is_up = false;
    search_data.nodes = 0;
    search_data.root_move_nodes = [[0; 49]; 49];

    let mut score: i16 = 0;

    // ID (Iterative deepening)
    for iteration_depth in 1..=search_data.max_depth 
    {
        let iteration_score = pvs(search_data, iteration_depth as i16, 0, -INFINITY, INFINITY);

        if search_data.is_hard_time_up() { break; }

        if print_info {
            println!("info depth {} score {} time {} nodes {} nps {} pv {}",
                    iteration_depth, 
                    iteration_score,
                    milliseconds_elapsed(search_data.start_time), 
                    search_data.nodes,
                    search_data.nodes * 1000 / milliseconds_elapsed(search_data.start_time).max(1) as u64,
                    move_to_str(search_data.best_move_root));
        }
        
        score = iteration_score;

        if search_data.is_soft_time_up() { break; }
    }

    assert!(search_data.best_move_root != MOVE_NONE);
    (search_data.best_move_root, score)
}

/*
fn aspiration(search_data: &mut SearchData, iteration_depth: u8, mut score: i16) -> i16
{
    let mut delta: i16 = 80;
    let mut alpha: i16 = (score - delta).max(-INFINITY);
    let mut beta: i16 = (score + delta).min(INFINITY);
    let mut depth: i16 = iteration_depth as i16;

    loop
    {
        score = pvs(search_data, depth, 0, -INFINITY, INFINITY);

        if search_data.is_hard_time_up() { return 0; }

        if score >= beta {
            beta = (beta + delta).min(INFINITY);
            depth -= 1;
        }
        else if score <= alpha {
            beta = (alpha + beta) / 2;
            alpha = (alpha - delta).max(-INFINITY);
            depth = iteration_depth as i16;
        }
        else {
            break;
        }

        delta *= 2;
    }

    score
}
*/

fn pvs(search_data: &mut SearchData, mut depth: i16, ply: i16, mut alpha: i16, beta: i16) -> i16
{
    if search_data.is_hard_time_up() { return 0; }

    if ply > 0
    {
        let game_result: GameResult = search_data.board.get_game_result();
        if game_result == GameResult::Draw {
            return 0;
        }
        else if game_result == GameResult::WinRed {
            return if search_data.board.state.color == Color::Red {INFINITY - ply} else {-INFINITY + ply};
        }
        else if game_result == GameResult::WinBlue {
            return if search_data.board.state.color == Color::Blue {INFINITY - ply} else {-INFINITY + ply};
        }
    }

    if depth <= 0 { 
        return evaluate(search_data.board.state.color, &search_data.board.state.accumulator); 
    }

    if depth > search_data.max_depth.into() { 
        depth = search_data.max_depth as i16; 
    }

    // Probe TT
    let tt_entry_index = search_data.board.state.zobrist_hash as usize % search_data.tt.entries.len();
    let tt_entry_probed: &TTEntry = &search_data.tt.entries[tt_entry_index];
    let tt_hit: bool = search_data.board.state.zobrist_hash == tt_entry_probed.zobrist_hash;
    let bound: Bound = tt_entry_probed.get_bound();

    // TT cutoff
    if ply > 0 && tt_hit && tt_entry_probed.depth >= (depth as u8)
    && (bound == Bound::Exact
    || (bound == Bound::Lower && tt_entry_probed.score >= beta)
    || (bound == Bound::Upper && tt_entry_probed.score <= alpha))
    {
        return tt_entry_probed.adjusted_score(ply);
    }

    let pv_node: bool = (beta as i32 - alpha as i32) > 1 || ply == 0;
    let eval = evaluate(search_data.board.state.color, &search_data.board.state.accumulator);

    if !pv_node
    {
        // RFP (Reverse futility pruning)
        if depth <= 5 && eval >= beta + depth * 50 {
            return eval;
        }
    }

    let mut moves: MovesArray = EMPTY_MOVES_ARRAY;
    let num_moves = search_data.board.moves(&mut moves);
    assert!(num_moves > 0);
    let tt_move = if tt_hit { search_data.tt.entries[tt_entry_index].get_move() }
                  else {MOVE_NONE};

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
        let (mov, move_score) = incremental_sort(&mut moves, num_moves, &mut moves_scores, i as usize);

        if ply > 0 && best_score > -MIN_WIN_SCORE
        {
            // FP (Futility pruning)
            if depth <= 5 && alpha < MIN_WIN_SCORE && i >= 3
            && eval + 40 + depth * 20 <= alpha {
                break;
            }
        }
        
        search_data.board.make_move(mov);
        search_data.nodes += 1;
        let nodes_before = search_data.nodes;

        // PVS (Principal variation search)
        let score = if i == 0 {
            -pvs(search_data, depth - 1, ply + 1, -beta, -alpha)
        } else {
            // LMR (Late move reductions)
            let lmr: i16 = if depth >= 3 && i >= 2 {
                let mut value: i16 = search_data.lmr_table[depth as usize][i as usize] as i16;
                value -= pv_node as i16; // reduce pv nodes less
                clamp(value, 0, depth - 2) // dont extend and dont reduce into eval
            } else {
                0
            };

            let null_window_score = -pvs(search_data, depth - 1 - lmr, ply + 1, -alpha - 1, -alpha);
            if null_window_score > alpha && (null_window_score < beta || lmr > 0) {
                -pvs(search_data, depth - 1, ply + 1, -beta, -alpha)
            } else {
                null_window_score
            }
        };

        search_data.board.undo_move();

        if ply == 0 && mov != MOVE_PASS {
            search_data.root_move_nodes[mov[FROM] as usize][mov[TO] as usize] 
                += search_data.nodes - nodes_before;
        }

        if search_data.is_hard_time_up() {
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
    tt_entry.zobrist_hash = search_data.board.state.zobrist_hash;
    tt_entry.depth = depth as u8;

    tt_entry.score = if best_score >= MIN_WIN_SCORE { best_score + ply }
                     else if best_score <= -MIN_WIN_SCORE { best_score - ply }
                     else { best_score };

    tt_entry.store_move_and_bound(best_move, if best_score <= original_alpha { Bound::Upper }
                                             else if best_score >= beta { Bound::Lower }
                                             else { Bound::Exact });

    best_score
}
