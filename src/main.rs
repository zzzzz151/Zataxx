#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

mod types;
mod utils;
mod board;
mod perft;
use types::*;
use utils::*;
use board::*;
use perft::*;

fn main() {
    #![allow(dead_code)]
    #![allow(unused_variables)]
    #![allow(unused_imports)]

    println!("Zataxx by zzzzz");
    init_attacks();

    let board: Board = Board::new(START_FEN2);
    board.print();

    run_perft_tests();

}
