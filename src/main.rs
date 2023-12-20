mod types;
mod utils;
mod tables;
mod nnue;
mod board;
mod perft;
mod tt;
mod datagen;
mod uai;
mod search;
mod tests;

use std::env;
use types::*;
//use utils::*;
use board::*;
//use perft::*;
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

    let mut search_data = SearchData::new(Board::new(START_FEN), 100, U64_MAX, U64_MAX, U64_MAX);
    uai_loop(&mut search_data);
}
