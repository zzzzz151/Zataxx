mod types;
mod utils;
mod tables;
mod board;
mod perft;
mod tt;
mod uai;
mod search;
//use types::*;
//use utils::*;
//use board::*;
//use perft::*;
use tt::*;
use uai::*;

fn main() {
    println!("Zataxx by zzzzz");
    let mut tt: TT = TT::new(DEFAULT_TT_SIZE_MB);
    uai_loop(&mut tt);
}
