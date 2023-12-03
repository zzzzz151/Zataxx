#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

mod types;
mod utils;
mod board;
mod perft;
mod uai;
mod search;
use types::*;
use utils::*;
use board::*;
use perft::*;
use uai::*;

fn main() {
    #![allow(dead_code)]
    #![allow(unused_variables)]
    #![allow(unused_imports)]

    println!("Zataxx by zzzzz");
    init_attacks();
    uai_loop();

}
