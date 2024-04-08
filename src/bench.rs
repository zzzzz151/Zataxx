use crate::types::*;
use crate::board::*;
use crate::search::*;
use crate::uai::*;

pub const DEFAULT_BENCH_DEPTH: u8 = 16;

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

    let mut searcher = Searcher::new(Board::new(START_FEN));
    let mut nodes: u64 = 0;
    let mut milliseconds: u64 = 0;

    for &fen in FENS.iter() 
    {
        searcher.board = Board::new(fen);
        assert!(searcher.board.game_state().0 == GameState::Ongoing);

        searcher.search(depth, I64_MAX, 0, true, U64_MAX, U64_MAX, false);
        milliseconds += searcher.milliseconds_elapsed();
        nodes += searcher.get_nodes();

        uainewgame(&mut searcher);
    }

    println!("bench depth {} nodes {} nps {} time {}", 
        depth, nodes, nodes * 1000 / milliseconds, milliseconds);
}