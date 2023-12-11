mod types;
mod utils;
mod tables;
mod board;
mod perft;
mod tt;
mod datagen;
mod uai;
mod search;
mod tests;

use std::time::Instant;
use types::*;
//use utils::*;
use tables::*;
use board::*;
//use perft::*;
use tt::*;
use search::*;
use uai::*;

fn main() {
    println!("Zataxx by zzzzz");

    let mut search_data = SearchData {
        board: Board::new(START_FEN),
        max_depth: MAX_DEPTH,
        start_time: Instant::now(),
        milliseconds: 4294967295,
        turn_milliseconds: 0,
        best_move_root: MOVE_NONE,
        nodes: 0,
        tt: TT::new(DEFAULT_TT_SIZE_MB),
        lmr_table: get_lmr_table()
    };

    uai_loop(&mut search_data);
}
