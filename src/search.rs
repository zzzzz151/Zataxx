use std::time::Instant;
use crate::tables::*;
use crate::types::*;
use crate::utils::*;
use crate::ataxx_move::*;
use crate::board::*;
use crate::tt::*;

pub struct SearchData {
    pub board: Board,
    pub max_depth: u8,
    pub max_ply_reached: u8,
    pub best_move_root: AtaxxMove,
    pub start_time: Instant,
    pub milliseconds: u64,
    pub time_is_up: bool,
    pub nodes: u64,
    pub root_move_nodes: [u64; 1usize << 13],
    pub soft_nodes: u64,
    pub hard_nodes: u64,
    pub tt: TT,
    pub lmr_table: [[u8; 256]; 256],
    pub killers: [AtaxxMove; 256],
}

impl SearchData
{
    pub fn new(board: Board, max_depth: u8, milliseconds: u64, soft_nodes: u64, hard_nodes: u64) -> Self {
        Self {
            board: board,
            max_depth: max_depth,
            max_ply_reached: 0,
            best_move_root: MOVE_NONE,
            start_time: Instant::now(),
            milliseconds: milliseconds,
            time_is_up: false,
            nodes: 0,
            root_move_nodes: [0; 1usize << 13],
            soft_nodes: soft_nodes,
            hard_nodes: hard_nodes,
            tt: TT::new(DEFAULT_TT_SIZE_MB),
            lmr_table: get_lmr_table(),
            killers: [MOVE_NONE; 256],
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
        } 
        else {
            let best_move_nodes = self.root_move_nodes[self.best_move_root.to_u16() as usize];
            best_move_nodes as f64 / self.nodes.max(1) as f64
        };

        let soft_time_scale = (0.5 + 1.0 - move_nodes_fraction) * 1.5;

        milliseconds_elapsed(self.start_time) 
        >= ((self.milliseconds as f64 * 0.05 * soft_time_scale) as u64)
    }
}

pub fn search(search_data: &mut SearchData, print_info: bool) -> (AtaxxMove, i32)
{
    search_data.best_move_root = MOVE_NONE;
    search_data.time_is_up = false;
    search_data.nodes = 0;
    search_data.root_move_nodes = [0; 1usize << 13];

    let mut score: i32 = 0;

    // ID (Iterative deepening)
    for iteration_depth in 1..=search_data.max_depth 
    {
        search_data.max_ply_reached = 0;
        let iteration_score = pvs(search_data, iteration_depth as i32, 0, -INFINITY, INFINITY, EVAL_NONE);

        if search_data.is_hard_time_up() { break; }

        if print_info {
            println!("info depth {} seldepth {} score {} time {} nodes {} nps {} pv {}",
                    iteration_depth, 
                    search_data.max_ply_reached,
                    iteration_score,
                    milliseconds_elapsed(search_data.start_time), 
                    search_data.nodes,
                    search_data.nodes * 1000 / milliseconds_elapsed(search_data.start_time).max(1) as u64,
                    search_data.best_move_root);
        }
        
        score = iteration_score;

        if search_data.is_soft_time_up() { break; }
    }

    assert!(search_data.best_move_root != MOVE_NONE);
    (search_data.best_move_root, score)
}

/*
fn aspiration(search_data: &mut SearchData, iteration_depth: u8, mut score: i32) -> i32
{
    let mut delta: i32 = 80;
    let mut alpha: i32 = (score - delta).max(-INFINITY);
    let mut beta: i32 = (score + delta).min(INFINITY);
    let mut depth: i32 = iteration_depth as i32;

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
            depth = iteration_depth as i32;
        }
        else {
            break;
        }

        delta *= 2;
    }

    score
}
*/

fn pvs(search_data: &mut SearchData, mut depth: i32, ply: i32, 
       mut alpha: i32, beta: i32, mut eval: i32) -> i32
{
    if search_data.is_hard_time_up() { return 0; }

    // Update seldepth
    if ply as u8 > search_data.max_ply_reached {
        search_data.max_ply_reached = ply as u8;
    }

    let singular: bool = eval != EVAL_NONE;

    if ply > 0 && !singular
    {
        let game_result: GameResult = search_data.board.get_game_result();
        if game_result == GameResult::Draw {
            return 0;
        }
        else if game_result == GameResult::WinRed 
        {
            return if search_data.board.state.color == Color::Red 
                {INFINITY - ply} 
            else 
                {-INFINITY + ply};
        }
        else if game_result == GameResult::WinBlue 
        {
            return if search_data.board.state.color == Color::Blue 
                {INFINITY - ply} 
            else 
                {-INFINITY + ply};
        }
    }

    if depth <= 0 || ply >= search_data.max_depth.into() { 
        return search_data.board.evaluate();
    }

    if depth > search_data.max_depth.into() { 
        depth = search_data.max_depth as i32; 
    }

    // Probe TT
    let tt_entry_index = search_data.board.state.zobrist_hash as usize % search_data.tt.entries.len();
    let tt_entry: TTEntry = search_data.tt.entries[tt_entry_index];
    let tt_hit: bool = search_data.board.state.zobrist_hash == tt_entry.zobrist_hash;
    let bound: Bound = tt_entry.get_bound();

    // TT cutoff
    if ply > 0 && !singular && tt_hit 
    && tt_entry.depth >= (depth as u8)
    && (bound == Bound::Exact
    || (bound == Bound::Lower && tt_entry.score >= beta as i16)
    || (bound == Bound::Upper && tt_entry.score <= alpha as i16))
    {
        return tt_entry.adjusted_score(ply as u8) as i32;
    }

    let pv_node: bool = beta - alpha > 1;
    if eval == EVAL_NONE {
        eval = search_data.board.evaluate();
    }

    // RFP (Reverse futility pruning)
    if !pv_node && !singular && depth <= 6 
    && eval >= beta + depth * 75 {
        return eval;
    }

    let tt_move = if tt_hit { tt_entry.get_move() } else { MOVE_NONE };

    // IIR (Internal iterative reduction)
    if tt_move == MOVE_NONE && depth >= 3 {
        depth -= 1;
    }

    // Generate moves
    let mut moves: MovesList = MovesList::default();
    search_data.board.moves(&mut moves);

    // Score moves
    let mut moves_scores: [u8; 256] = [0; 256];
    if moves.num_moves > 1 {
        for i in 0..(moves.num_moves as usize) { 
            let mov: AtaxxMove = moves[i];
            if mov == tt_move {
                moves_scores[i] = 255;
            }
            else {
                moves_scores[i] = mov.is_single() as u8;
                moves_scores[i] += search_data.board.num_adjacent_enemies(mov.to);
                moves_scores[i] += (mov == search_data.killers[ply as usize]) as u8 * 2;
            }
        }
    }

    let mut best_score: i32 = -INFINITY;
    let mut best_move: AtaxxMove = MOVE_NONE;
    let original_alpha = alpha;

    for i in 0..(moves.num_moves as usize)
    {
        let (mov, move_score) = incremental_sort(&mut moves, &mut moves_scores, i);

        // Don't search the excluded TT move in a singular search
        if mov == tt_move && singular { continue; }

        if ply > 0 && best_score > -MIN_WIN_SCORE
        {
            // LMP (Late move pruning)
            if move_score == 0 && i >= 2 {
                break;
            }

            // FP (Futility pruning)
            if depth <= 6 && alpha < MIN_WIN_SCORE && i >= 3
            && eval + 160 + depth * 80 <= alpha {
                break;
            }
        }
                
        // SE (Singular extensions)
        let mut extension: i32 = 0;
        if mov == tt_move && !singular 
        && !pv_node && depth >= 6
        && tt_entry.score.abs() < MIN_WIN_SCORE as i16
        && tt_entry.depth as i32 >= depth - 3
        && bound != Bound::Upper
        {
            let singular_beta: i32 = (tt_entry.score as i32 - depth).max(-INFINITY);
            let singular_score = pvs(search_data, (depth - 1) / 2, ply, 
                                     singular_beta - 1, singular_beta, eval);

            if singular_score < singular_beta {
                extension = 1;
            }
            // Multicut
            else if singular_beta >= beta {
                return singular_beta;
            }
            // Negative extenesion
            else if tt_entry.score >= beta as i16 {
                extension = -1;
            }
        }

        search_data.board.make_move(mov);
        let nodes_before = search_data.nodes;
        search_data.nodes += 1;

        // PVS (Principal variation search)
        let score = if i == 0 {
            -pvs(search_data, depth - 1 + extension, ply + 1,
                 -beta, -alpha, EVAL_NONE)
        } else {
            // LMR (Late move reductions)
            let lmr: i32 = if depth >= 3 && i >= 2 {
                let mut value: i32 = search_data.lmr_table[depth as usize][i as usize] as i32;
                value -= pv_node as i32; // reduce pv nodes less
                clamp(value, 0, depth - 2) // dont extend and dont reduce into eval
            } else {
                0
            };

            let null_window_score = -pvs(search_data, depth - 1 - lmr, ply + 1, 
                                         -alpha - 1, -alpha, EVAL_NONE);

            if null_window_score > alpha && (null_window_score < beta || lmr > 0) {
                -pvs(search_data, depth - 1, ply + 1, -beta, -alpha, EVAL_NONE)
            } else {
                null_window_score
            }
        };

        search_data.board.undo_move();

        if ply == 0 && mov != MOVE_PASS {
            search_data.root_move_nodes[mov.to_u16() as usize]
                += search_data.nodes - nodes_before;
        }

        if search_data.is_hard_time_up() { return 0; }

        if score > best_score { best_score = score; }

        if score <= alpha { continue; } // Fail low

        alpha = score;
        best_move = mov;
        if ply == 0 { 
            search_data.best_move_root = mov; 
        }

        if score < beta { continue; }

        // Fail high / beta cutoff

        search_data.killers[ply as usize] = mov;

        break; // Fail high / beta cutoff
    }

    if !singular
    {
        let tt_entry: &mut TTEntry = &mut search_data.tt.entries[tt_entry_index];
        tt_entry.zobrist_hash = search_data.board.state.zobrist_hash;
        tt_entry.depth = depth as u8;

        tt_entry.score = if best_score >= MIN_WIN_SCORE 
            { (best_score + ply) as i16 }
        else if best_score <= -MIN_WIN_SCORE 
            { (best_score - ply) as i16 }
        else 
            { best_score as i16 };

        if best_move != MOVE_NONE {
            tt_entry.set_move(best_move);
        }

        tt_entry.set_bound(if best_score <= original_alpha 
                               { Bound::Upper }
                           else if best_score >= beta
                               { Bound::Lower }
                           else 
                               { Bound::Exact });
    }

    best_score
}
