mod types;
mod utils;
mod tables;
mod board;
mod perft;
mod tt;
mod uai;
mod search;
mod tests;

use std::time::Instant;
use types::*;
//use utils::*;
use board::*;
//use perft::*;
use tt::*;
use search::*;
use uai::*;

fn main() {
    println!("Zataxx by zzzzz");

    let mut search_data = SearchData {
        board: Board::new(START_FEN),
        start_time: Instant::now(),
        milliseconds: 0,
        turn_milliseconds: 0,
        best_move_root: MOVE_NONE,
        tt: TT::new(DEFAULT_TT_SIZE_MB)
    };

    uai_loop(&mut search_data);
}
