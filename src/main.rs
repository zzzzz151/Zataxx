mod types;
mod utils;
mod tables;
mod nnue;
mod ataxx_move;
mod board;
mod perft;
mod tt_entry;
mod datagen;
mod uai;
mod search;
mod bench;
mod tests;

use std::env;
use types::*;
//use utils::*;
use board::*;
//use perft::*;
use search::*;
use uai::*;
use datagen::*;

#[allow(unreachable_code)]

fn main() {
    println!("Zataxx by zzzzz");

    #[cfg(target_feature="avx2")] {  
        println!("Using avx2");
    }

    #[cfg(not(target_feature="avx2"))] {
        println!("Warning: not using avx2");
    }

    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        let arg = args[1].trim();
        if arg == "datagen" {
            datagen();
            return;
        }
        else if arg == "datagen_openings"
        {
            datagen_openings();
            return;
        }
    }

    let mut searcher = Searcher::new(Board::new(START_FEN));
    uai_loop(&mut searcher);
}
