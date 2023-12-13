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
use std::env;
use types::*;
//use utils::*;
use tables::*;
use board::*;
//use perft::*;
use tt::*;
use search::*;
use uai::*;
use datagen::*;

fn main() {
    println!("Zataxx by zzzzz");

    let args: Vec<String> = env::args().collect();
    if args.len() > 1 && args[1] == "datagen" {
        datagen();
        return;
    }

    let mut search_data = SearchData {
        board: Board::new(START_FEN),
        max_depth: 100,
        start_time: Instant::now(),
        milliseconds: 4294967295,
        turn_milliseconds: 0,
        time_is_up: false,
        soft_nodes: 4294967295,
        hard_nodes: 4294967295,
        best_move_root: MOVE_NONE,
        nodes: 0,
        tt: TT::new(DEFAULT_TT_SIZE_MB),
        lmr_table: get_lmr_table()
    };

    uai_loop(&mut search_data);
}
