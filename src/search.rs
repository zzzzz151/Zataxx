use std::time::Instant;
use crate::tables::*;
use crate::types::*;
use crate::utils::*;
use crate::ataxx_move::*;
use crate::board::*;
use crate::tt_entry::*;

pub const DEFAULT_MAX_DEPTH: u8 = 100;
pub const DEFAULT_TT_SIZE_MB: usize = 32;

pub struct Searcher {
    pub board: Board,
    pub max_depth: u8,
    pub max_ply_reached: u8,
    pub start_time: Instant,
    pub milliseconds: u64,
    pub soft_nodes: u64,
    pub hard_nodes: u64,
    pub nodes: u64,
    pub best_move_root: AtaxxMove,
    root_move_nodes: [u64; 1usize << 13],
    evals: Vec<i32>,
    tt: Vec<TTEntry>,
    lmr_table: Vec<Vec<u8>>,
    killers: Vec<AtaxxMove>
}

impl Searcher
{
    pub fn new(board: Board) -> Self 
    {
        let mut searcher = Self {
            board: board,
            max_depth: DEFAULT_MAX_DEPTH,
            max_ply_reached: 0,
            start_time: Instant::now(),
            milliseconds: U64_MAX,
            soft_nodes: U64_MAX,
            hard_nodes: U64_MAX,
            nodes: 0,
            best_move_root: MOVE_NONE,
            root_move_nodes: [0; 1usize << 13],
            evals: vec![0; DEFAULT_MAX_DEPTH as usize],
            tt: vec![TTEntry::default(); 0],
            lmr_table: get_lmr_table(DEFAULT_MAX_DEPTH),
            killers: vec![MOVE_NONE; DEFAULT_MAX_DEPTH as usize]
        };

        searcher.resize_tt(DEFAULT_TT_SIZE_MB);
        searcher
    }

    pub fn resize_tt(&mut self, size_mb: usize) 
    {
        let num_entries: usize = (size_mb * 1024 * 1024 / std::mem::size_of::<TTEntry>()) as usize;
        self.tt = vec![TTEntry::default(); num_entries];
        assert!(self.tt.len() == num_entries);
        println!("TT size: {} ({} entries)", size_mb, num_entries);
    }

    pub fn clear_tt(&mut self) { 
        self.tt = vec![TTEntry::default(); self.tt.len()];
    }

    pub fn clear_killers(&mut self) {
        self.killers = vec![MOVE_NONE; self.killers.len()];
    }

    pub fn is_hard_time_up(&self) -> bool 
    {
        if self.nodes >= self.hard_nodes { 
            return true; 
        }

        if (self.nodes % 1024) != 0 {
            return false;
        }

        milliseconds_elapsed(self.start_time) >= (self.milliseconds / 2)
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
            let best_move_nodes = self.root_move_nodes[self.best_move_root.to_u12() as usize];
            best_move_nodes as f64 / self.nodes.max(1) as f64
        };

        let soft_time_scale = (0.5 + 1.0 - move_nodes_fraction) * 1.5;

        milliseconds_elapsed(self.start_time) 
        >= ((self.milliseconds as f64 * 0.05 * soft_time_scale) as u64)
    }

    pub fn search(&mut self, print_info: bool) -> (AtaxxMove, i32)
    {
        self.nodes = 0;
        self.root_move_nodes = [0; 1usize << 13];
        self.best_move_root = MOVE_NONE;

        let mut score: i32 = 0;

        // ID (Iterative deepening)
        for iteration_depth in 1..=self.max_depth 
        {
            self.max_ply_reached = 0;
            let iteration_score = self.pvs(iteration_depth as i32, 0, -INFINITY, INFINITY, false);

            if self.is_hard_time_up() { break; }

            if print_info {
                println!("info depth {} seldepth {} score {} time {} nodes {} nps {} pv {}",
                    iteration_depth, 
                    self.max_ply_reached,
                    iteration_score,
                    milliseconds_elapsed(self.start_time), 
                    self.nodes,
                    self.nodes * 1000 / milliseconds_elapsed(self.start_time).max(1) as u64,
                    self.best_move_root);
            }
            
            score = iteration_score;

            if self.is_soft_time_up() { break; }
        }

        assert!(self.best_move_root != MOVE_NONE);
        (self.best_move_root, score)
    }

    /*
    fn aspiration(self: &mut SearchData, iteration_depth: u8, mut score: i32) -> i32
    {
        let mut delta: i32 = 80;
        let mut alpha: i32 = (score - delta).max(-INFINITY);
        let mut beta: i32 = (score + delta).min(INFINITY);
        let mut depth: i32 = iteration_depth as i32;

        loop
        {
            score = self.pvs(depth, 0, -INFINITY, INFINITY);

            if self.is_hard_time_up() { return 0; }

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

    fn pvs(&mut self, mut depth: i32, ply: i32, 
           mut alpha: i32, beta: i32, singular: bool) -> i32
    {
        if self.is_hard_time_up() { return 0; }

        // Update seldepth
        if ply as u8 > self.max_ply_reached {
            self.max_ply_reached = ply as u8;
        }

        if ply > 0 && !singular {
            let (game_state, winner): (GameState, Color) = self.board.game_state();

            if game_state == GameState::Draw { return 0 };

            if winner != Color::None {
                return if winner == self.board.state.color {
                    INFINITY - ply
                } else {
                    -INFINITY + ply
                }
            }
        }

        if depth <= 0 || ply >= self.max_depth.into() { 
            return self.board.evaluate();
        }

        if depth > self.max_depth.into() { 
            depth = self.max_depth as i32; 
        }

        // Probe TT
        let tt_entry_index = self.board.state.zobrist_hash as usize % self.tt.len();
        let tt_entry: &TTEntry = &self.tt[tt_entry_index];
        let tt_hit: bool = self.board.state.zobrist_hash == tt_entry.zobrist_hash;

        // TT cutoff
        if tt_hit && ply > 0 && !singular
        && tt_entry.depth >= (depth as u8)
        && (tt_entry.get_bound() == Bound::Exact
        || (tt_entry.get_bound() == Bound::Lower && tt_entry.score >= beta as i16)
        || (tt_entry.get_bound() == Bound::Upper && tt_entry.score <= alpha as i16))
        {
            return tt_entry.adjusted_score(ply as u8) as i32;
        }

        let pv_node = beta - alpha > 1;
        let eval = if singular { self.evals[ply as usize] } else { self.board.evaluate() };
        //self.evals[ply as usize] = eval;

        // RFP (Reverse futility pruning)
        if !pv_node && !singular && depth <= 6 
        && eval >= beta + depth * 50 {
            return eval;
        }

        let tt_move = if tt_hit { tt_entry.get_move() } else { MOVE_NONE };

        // IIR (Internal iterative reduction)
        if tt_move == MOVE_NONE && depth >= 3 {
            depth -= 1;
        }

        // Generate moves
        let mut moves: MovesList = MovesList::default();
        self.board.moves(&mut moves);

        // Score moves
        let mut moves_scores: [u8; 256] = [0; 256];
        if moves.size() > 1 {
            for i in 0..(moves.size() as usize) { 
                let mov: AtaxxMove = moves[i];
                if mov == tt_move {
                    moves_scores[i] = 255;
                }
                else {
                    moves_scores[i] = if mov.is_single() { 100 } else { 0 };
                    moves_scores[i] += self.board.num_adjacent_enemies(mov.to);
                    moves_scores[i] += (mov == self.killers[ply as usize]) as u8;
                }
            }
        }

        let mut best_score: i32 = -INFINITY;
        let mut best_move: AtaxxMove = MOVE_NONE;
        let mut bound: Bound = Bound::Upper;
        //let improving = ply > 1 && eval > self.evals[ply as usize - 2];

        for i in 0..(moves.size() as usize)
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

            let tt_entry: &TTEntry = &self.tt[tt_entry_index];
                    
            // SE (Singular extensions)
            let mut extension: i32 = 0;
            if mov == tt_move && !singular 
            && !pv_node && depth >= 6
            && tt_entry.score.abs() < MIN_WIN_SCORE as i16
            && tt_entry.depth as i32 >= depth - 3
            && tt_entry.get_bound() != Bound::Upper
            {
                let singular_beta: i32 = (tt_entry.score as i32 - depth).max(-INFINITY);
                let singular_score = self.pvs((depth - 1) / 2, ply, singular_beta - 1, singular_beta, true);

                // Normal singular extension
                if singular_score < singular_beta {
                    extension = 1;
                }
                // Multicut
                else if singular_beta >= beta {
                    return singular_beta;
                }
                // Negative extension
                else if self.tt[tt_entry_index].score >= beta as i16 {
                    extension = -1;
                }
            }

            self.board.make_move(mov);
            let nodes_before = self.nodes;
            self.nodes += 1;

            // PVS (Principal variation search)
            let score = if i == 0 {
                -self.pvs(depth - 1 + extension, ply + 1,-beta, -alpha, false)
            } else {
                // LMR (Late move reductions)
                let lmr: i32 = if depth >= 3 && i >= 2 
                {
                    let mut value: i32 = self.lmr_table[depth as usize][i+1] as i32;
                    value -= pv_node as i32;
                    clamp(value, 0, depth - 2) // dont extend and dont reduce into eval
                } else {
                    0
                };

                let null_window_score = -self.pvs(depth - 1 - lmr, ply + 1, -alpha - 1, -alpha, false);

                if null_window_score > alpha && (pv_node || lmr > 0) {
                    -self.pvs(depth - 1, ply + 1, -beta, -alpha, false)
                } else {
                    null_window_score
                }
            };

            self.board.undo_move();

            if ply == 0 && mov != MOVE_PASS {
                self.root_move_nodes[mov.to_u12() as usize]
                    += self.nodes - nodes_before;
            }

            if self.is_hard_time_up() { return 0; }

            if score > best_score { best_score = score; }

            if score <= alpha { continue; } // Fail low

            bound = Bound::Exact;
            alpha = score;
            best_move = mov;

            if ply == 0 { self.best_move_root = mov; }

            if score < beta { continue; }

            // Fail high / beta cutoff

            bound = Bound::Lower;
            if mov != MOVE_PASS {
                self.killers[ply as usize] = mov;
            }

            break; // Fail high / beta cutoff
        }

        // Store in TT
        if !singular
        {
            let tt_entry: &mut TTEntry = &mut self.tt[tt_entry_index];
            tt_entry.zobrist_hash = self.board.state.zobrist_hash;
            tt_entry.depth = depth as u8;

            tt_entry.score = if best_score >= MIN_WIN_SCORE { 
                (best_score + ply) as i16 
            } else if best_score <= -MIN_WIN_SCORE { 
                (best_score - ply) as i16 
            } else { 
                best_score as i16 
            };

            if best_move != MOVE_NONE {
                tt_entry.set_move(best_move);
            }

            tt_entry.set_bound(bound);
        }

        best_score
    }
}
