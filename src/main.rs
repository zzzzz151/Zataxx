mod types;
mod utils;
mod tables;
mod nnue;
mod ataxx_move;
mod board;
mod perft;
mod tt;
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
        panic!("Not using avx2!");
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

    let mut search_data = SearchData::new(Board::new(START_FEN), 100, 
                                          U64_MAX, U64_MAX, U64_MAX);
    uai_loop(&mut search_data);
}
