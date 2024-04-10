use std::time::Instant;
use crate::types::*;
use crate::utils::*;
use crate::ataxx_move::*;
use arrayvec::ArrayVec;
use crate::nnue::*;
use crate::board::*;
use crate::tt_entry::*;
use crate::tunable_params;

pub const DEFAULT_MAX_DEPTH: u8 = 100;
pub const TT_DEFAULT_MB: usize = 32;

tunable_params! {
    rfp_max_depth: i32 = 6, 6, 10, 1;
    rfp_multiplier: i32 = 50, 20, 160, 10;
    iir_min_depth: i32 = 3, 3, 6, 1;
    fp_max_depth: i32 = 6, 6, 10, 1;
    fp_min_moves: usize = 3, 2, 8, 1;
    fp_base: i32 = 160, 20, 260, 20;
    fp_multiplier: i32 = 80, 20, 260, 20;
    singular_min_depth: i32 = 6, 6, 10, 1;
    singular_depth_margin: i32 = 3, 1, 5, 1;
    lmr_base: f64 = 0.8, 0.1, 2.0, 0.1;
    lmr_multiplier: f64 = 0.4, 0.1, 2.0, 0.1;
    lmr_min_moves: usize = 2, 2, 8, 1;
    lmr_history_div: f32 = 12288.0, 2048.0, 24576.0, 2048.0;
    history_bonus_multiplier: i32 = 300, 50, 600, 50;
    history_bonus_max: i32 = 1500, 500, 2500, 100;
    history_malus_multiplier: i32 = 300, 50, 600, 50;
    history_malus_max: i32 = 1500, 500, 2500, 100;
    history_max: i32 = 16384, 4096, 32768, 4096;
    hard_time_percentage: f64 = 0.5, 0.3, 0.7, 0.1;
    soft_time_percentage: f64 = 0.05, 0.03, 0.07, 0.02;
    soft_time_scale: f64 = 0.6, 0.4, 0.8, 0.1;
    nodes_tm_min_depth: u8 = 7, 7, 11, 1;
    nodes_tm_base: f64 = 1.55, 1.3, 1.7, 0.2;
    nodes_tm_multiplier: f64 = 1.5, 1.3, 1.7, 0.2;
}

pub struct Searcher {
    pub board: Board,
    pub accumulator: Accumulator,
    max_depth: u8,
    max_ply_reached: u8,
    nodes: u64,
    start_time: Instant,
    hard_milliseconds: u64,
    soft_milliseconds: u64,
    hard_nodes: u64,
    soft_nodes: u64,
    best_move_root: AtaxxMove,
    root_move_nodes: [u64; 1usize << 13],
    tt: Vec<TTEntry>,
    evals: Vec<i32>,
    lmr_table: [[u8; 256]; 256],
    killers: Vec<AtaxxMove>,
    history: [[[i32; 49]; 49]; 2], // [color][move.from][move.to]
}

impl Searcher
{
    pub fn new(board: Board) -> Self 
    {
        let mut searcher = Self {
            board: board.clone(),
            accumulator: Accumulator::new(board.bitboards()),
            max_depth: DEFAULT_MAX_DEPTH,
            max_ply_reached: 0,
            nodes: 0,
            start_time: Instant::now(),
            hard_milliseconds: U64_MAX,
            soft_milliseconds: U64_MAX,
            hard_nodes: U64_MAX,
            soft_nodes: U64_MAX,
            best_move_root: MOVE_NONE,
            root_move_nodes: [0; 1usize << 13],
            tt: vec![TTEntry::default(); 0],
            evals: vec![0; 256],
            lmr_table: [[0; 256]; 256],
            killers: vec![MOVE_NONE; 256 as usize],
            history: [[[0; 49]; 49]; 2],
        };

        searcher.init_lmr_table();
        searcher.resize_tt(TT_DEFAULT_MB);
        searcher
    }

    pub fn init_lmr_table(&mut self) {
        for depth in 1..256 {
            for move_num in 1..256 
            {
                let value: f64 = lmr_base() + (depth as f64).ln() * (move_num as f64).ln() * lmr_multiplier();
                self.lmr_table[depth][move_num] = value.round().clamp(0.0, 255.0) as u8;
            }
        }
    }

    pub fn resize_tt(&mut self, size_mb: usize)
    {
        let num_entries: usize = (size_mb * 1024 * 1024 / std::mem::size_of::<TTEntry>()) as usize;
        self.tt = vec![TTEntry::default(); num_entries];
    }

    pub fn print_tt_size(&self) {
        let size_bytes: usize = self.tt.len() * std::mem::size_of::<TTEntry>();
        let size_mb: f64 = size_bytes as f64 / (1024.0 * 1024.0);
        println!("TT size: {} MB ({} entries)", size_mb.round(), self.tt.len());    
    }

    pub fn get_nodes(&self) -> u64 { self.nodes }

    pub fn clear_tt(&mut self) { 
        self.tt = vec![TTEntry::default(); self.tt.len()];
    }

    pub fn clear_killers(&mut self) {
        self.killers = vec![MOVE_NONE; self.killers.len()];
    }

    pub fn clear_history(&mut self) {
        self.history = [[[0; 49]; 49]; 2];
    }

    pub fn milliseconds_elapsed(&self) -> u64 {
        milliseconds_elapsed(self.start_time)
    }

    pub fn is_hard_time_up(&self) -> bool 
    {
        if self.best_move_root == MOVE_NONE {
            return false;
        }

        if self.nodes >= self.hard_nodes { 
            return true; 
        }

        if (self.nodes % 1024) != 0 {
            return false;
        }

        self.milliseconds_elapsed() >= self.hard_milliseconds
    }

    pub fn search(&mut self, max_depth: u8, milliseconds: i64, increment_ms: u64, is_move_time: bool, 
        soft_nodes: u64, hard_nodes: u64, print_info: bool) -> (AtaxxMove, i32)
    {
        // init/reset stuff
        self.start_time = Instant::now();
        self.max_depth = max_depth;
        self.soft_nodes = soft_nodes;
        self.hard_nodes = hard_nodes;
        self.nodes = 0;
        self.best_move_root = MOVE_NONE;
        self.root_move_nodes = [0; 1usize << 13];

        // Set time limits
        let max_hard_ms: u64 = (milliseconds - 10).max(0) as u64;
        if is_move_time {
            self.hard_milliseconds = max_hard_ms;
            self.soft_milliseconds = U64_MAX;
        }
        else {
            self.hard_milliseconds = (max_hard_ms as f64 * hard_time_percentage()) as u64;
            let mut soft_milliseconds: f64 = max_hard_ms as f64 * soft_time_percentage() + increment_ms as f64 * 0.6666;
            soft_milliseconds *= soft_time_scale();
            self.soft_milliseconds = (soft_milliseconds as u64).min(self.hard_milliseconds);
        }

        // ID (Iterative deepening)
        let mut score: i32 = 0;
        for iteration_depth in 1..=self.max_depth 
        {
            self.max_ply_reached = 0;
            let iteration_score = self.pvs(iteration_depth as i32, 0, -INFINITY, INFINITY, false);

            if self.is_hard_time_up() { break; }

            assert!(self.best_move_root != MOVE_NONE);

            let ms_elapsed = self.milliseconds_elapsed();

            if print_info {
                println!("info depth {} seldepth {} score {} time {} nodes {} nps {} pv {}",
                    iteration_depth, 
                    self.max_ply_reached,
                    iteration_score,
                    ms_elapsed, 
                    self.nodes,
                    self.nodes * 1000 / ms_elapsed.max(1),
                    self.best_move_root);
            }
            
            score = iteration_score;

            // Check soft nodes
            if self.nodes >= self.soft_nodes {
                break;
            }

            // Stop searching if soft time exceeded

            let updated_soft_milliseconds: u64 = if iteration_depth >= nodes_tm_min_depth()
            {
                let best_move_nodes_fraction: f64 = if self.best_move_root == MOVE_PASS { 
                    1.0 
                } 
                else {
                    self.root_move_nodes[self.best_move_root.to_u12() as usize] as f64 
                    / self.nodes.max(1) as f64
                };

                (self.soft_milliseconds as f64 
                * (nodes_tm_base() - best_move_nodes_fraction)
                * nodes_tm_multiplier())
                as u64
            }
            else {
                self.soft_milliseconds
            };

            if ms_elapsed >= updated_soft_milliseconds { 
                break; 
            }
        }

        assert!(self.best_move_root != MOVE_NONE);
        (self.best_move_root, score)
    }

    fn pvs(&mut self, mut depth: i32, ply: i32, 
           mut alpha: i32, beta: i32, singular: bool) -> i32
    {
        if self.is_hard_time_up() { return 0; }

        // Update seldepth
        if ply as u8 > self.max_ply_reached {
            self.max_ply_reached = ply as u8;
        }

        // Game over?
        if ply > 0 && !singular {
            let (game_state, winner): (GameState, Color) = self.board.game_state();

            if game_state == GameState::Draw { return 0 };

            if winner != Color::None {
                return if winner == self.board.side_to_move() {
                    INFINITY - ply
                } else {
                    -INFINITY + ply
                }
            }
        }

        // Leaf node, return static eval
        if depth <= 0 || ply >= self.max_depth.into() { 
            return evaluate(self.board.side_to_move(), &mut self.accumulator, self.board.bitboards());
        }

        if depth > self.max_depth.into() { 
            depth = self.max_depth as i32; 
        }

        // Probe TT
        let tt_entry_index = self.board.zobrist_hash() as usize % self.tt.len();
        let tt_entry: &TTEntry = &self.tt[tt_entry_index];
        let tt_hit: bool = self.board.zobrist_hash() == tt_entry.zobrist_hash;

        // TT cutoff
        if tt_hit 
        && ply > 0 
        && !singular
        && tt_entry.depth >= (depth as u8)
        && (tt_entry.get_bound() == Bound::Exact
        || (tt_entry.get_bound() == Bound::Lower && tt_entry.score >= beta as i16)
        || (tt_entry.get_bound() == Bound::Upper && tt_entry.score <= alpha as i16))
        {
            return tt_entry.adjusted_score(ply as u8) as i32;
        }

        let pv_node = beta - alpha > 1;

        let eval = if singular { 
            self.evals[ply as usize] 
        } else { 
            evaluate(self.board.side_to_move(), &mut self.accumulator, self.board.bitboards())
        };
        self.evals[ply as usize] = eval;

        // RFP (Reverse futility pruning)
        if !pv_node 
        && !singular 
        && depth <= rfp_max_depth()
        && eval >= beta + depth * rfp_multiplier() {
            return eval;
        }

        let tt_move = if tt_hit { tt_entry.get_move() } else { MOVE_NONE };

        // IIR (Internal iterative reduction)
        if tt_move == MOVE_NONE && depth >= iir_min_depth() {
            depth -= 1;
        }

        let stm: usize = self.board.side_to_move() as usize;

        // Generate moves
        let mut moves = ArrayVec::<AtaxxMove, 256>::new();
        self.board.moves(&mut moves);

        // Score moves
        let mut moves_scores = ArrayVec::<i32, 256>::new();
        if moves.len() > 1 {
            for i in 0..(moves.len() as usize) { 
                let mov: AtaxxMove = moves[i];
                if mov == tt_move {
                    moves_scores.push(I32_MAX);
                }
                else {
                    let mut move_score: i32 = mov.is_single() as i32 * 2;
                    move_score += self.board.num_adjacent_enemies(mov.to) as i32;
                    move_score *= 1_000_000;
                    move_score += self.history[stm][mov.from as usize][mov.to as usize];
                    moves_scores.push(move_score);
                }
            }
        }
        else {
            moves_scores.push(0);
        }

        let mut best_score: i32 = -INFINITY;
        let mut best_move: AtaxxMove = MOVE_NONE;
        let mut bound: Bound = Bound::Upper;
        //let improving = ply > 1 && eval > self.evals[ply as usize - 2];

        for i in 0..(moves.len() as usize)
        {
            let (mov, move_score) = incremental_sort(&mut moves, &mut moves_scores, i);

            // Don't search the excluded TT move in a singular search
            if mov == tt_move && singular { continue; }

            if ply > 0 && best_score > -MIN_WIN_SCORE
            {
                // LMP (Late move pruning)
                if move_score < history_max() && i >= 2 {
                    break;
                }

                // FP (Futility pruning)
                if depth <= fp_max_depth() 
                && alpha < MIN_WIN_SCORE 
                && i >= fp_min_moves()
                && eval + fp_base() + depth * fp_multiplier() <= alpha {
                    break;
                }
            }

            let tt_entry: &TTEntry = &self.tt[tt_entry_index];
                    
            // SE (Singular extensions)
            let mut extension: i32 = 0;
            if mov == tt_move 
            && !singular 
            && !pv_node 
            && depth >= singular_min_depth()
            && tt_entry.score.abs() < MIN_WIN_SCORE as i16
            && tt_entry.depth as i32 >= depth - singular_depth_margin()
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
                let lmr: i32 = if depth >= 3 && i >= lmr_min_moves() 
                {
                    let history: f32 = self.history[stm][mov.from as usize][mov.to as usize] as f32;

                    (self.lmr_table[depth as usize][i+1] as i32
                    // reduce pv nodes less
                    - pv_node as i32
                    // reduce moves with good history less and vice versa
                    - (history / lmr_history_div()).round() as i32)
                    // dont extend or drop into static eval
                    .clamp(0, depth - 2)
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

            // At root, update root moves nodes
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

            if mov == MOVE_PASS { break; }

            self.killers[ply as usize] = mov; // This move is now a killer move

            // Increase this move's history
            let mut move_history =  &mut self.history[stm][mov.from as usize][mov.to as usize];
            let bonus: i32 = (depth * history_bonus_multiplier()).min(history_bonus_max());
            *move_history += bonus - bonus * *move_history / history_max();

            // History malus: decrease history of tried moves
            let malus: i32 = (depth * history_malus_multiplier()).min(history_malus_max());
            for j in 0..i {
                move_history =  &mut self.history[stm][moves[j].from as usize][moves[j].to as usize];
                *move_history += -malus - malus * *move_history / history_max();
            }

            break;
        }

        // Store in TT
        if !singular
        {
            let tt_entry: &mut TTEntry = &mut self.tt[tt_entry_index];
            tt_entry.zobrist_hash = self.board.zobrist_hash();
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
