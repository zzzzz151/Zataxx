use std::time::Instant;
use crate::types::*;
use crate::utils::*;
use crate::board::*;
use crate::search::*;
use crate::uai::*;

const FENS: [&str; 17] = [
    "7/7/7/7/-------/-------/r5b r 0 1",
    "7/7/7/7/-------/-------/r5b b 0 1",
    "r5b/7/2-1-2/7/2-1-2/7/b5r r 0 1",
    "r5b/7/2-1-2/7/2-1-2/7/b5r b 0 1",
    "r5b/7/2-1-2/7/2-1-2/7/b5r r 0 1",
    "r5b/7/2-1-2/3-3/2-1-2/7/b5r b 0 1",
    "r5b/7/2-1-2/3-3/2-1-2/7/b5r r 0 1",
    "r5b/7/3-3/2-1-2/3-3/7/b5r b 0 1",
    "r5b/7/3-3/2-1-2/3-3/7/b5r r 0 1",
    "r5b/7/7/7/7/7/b5r r 0 1",
    "r5b/7/7/7/7/7/b5r b 0 1",
    "7/7/7/2r1b2/7/7/7 r 0 1",
    "7/7/7/2r1b2/7/7/7 b 0 1",
    "7/7/7/7/bbbbbbb/bbbbbbb/rrrrrrr r 0 1",
    "7/7/7/7/rrrrrrr/rrrrrrr/bbbbbbb b 0 1",
    "7/7/7/7/bbbbbbb/bbbbbbb/rrrrrrr b 0 1",
    "7/7/7/7/rrrrrrr/rrrrrrr/bbbbbbb r 0 1"
];

pub fn bench(depth: u8) {
    println!("Running bench depth {}", depth);

    let mut search_data = SearchData::new(Board::new(START_FEN), depth, 
                                          U64_MAX, U64_MAX, U64_MAX);

    let mut nodes: u64 = 0;
    let mut milliseconds: u64 = 0;

    for &fen in FENS.iter() 
    {
        search_data.board = Board::new(fen);
        assert!(search_data.board.get_game_result() == GameResult::None);

        search_data.start_time = Instant::now();
        search(&mut search_data, false);

        nodes += search_data.nodes;
        milliseconds += milliseconds_elapsed(search_data.start_time);

        uainewgame(&mut search_data);
    }

    println!("bench depth {} nodes {} nps {} time {}", 
             depth, nodes, nodes * 1000 / milliseconds, milliseconds);
}