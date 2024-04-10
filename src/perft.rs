use std::time::Instant;
use crate::board::*;
use crate::types::*;
use crate::utils::*;
use crate::ataxx_move::*;
use arrayvec::ArrayVec;

pub fn perft(board: &mut Board, depth: u8) -> u64
{
    if depth == 0 {
        return 1u64;
    }

    if board.game_state().0 != GameState::Ongoing {
        return 0u64;
    }

    // Generate moves
    let mut moves = ArrayVec::<AtaxxMove, 256>::new();
    board.moves(&mut moves);

    if depth == 1 { return moves.len() as u64 };

    let mut nodes: u64 = 0;

    for i in 0..(moves.len() as usize)
    {
        board.make_move(moves[i]);
        nodes += perft(board, depth - 1);
        board.undo_move();
    }

    nodes
}

pub fn perft_split(board: &mut Board, depth: u8)
{
    assert!(depth > 0);
    println!("Running split perft depth {} on {}", depth, board.fen());

    // Generate moves
    let mut moves = ArrayVec::<AtaxxMove, 256>::new();
    board.moves(&mut moves);
    let mut total_nodes: u64 = 0;

    for i in 0..(moves.len() as usize)
    {
        let mov: AtaxxMove = moves[i];
        board.make_move(mov);
        let nodes: u64 = perft(board, depth - 1);
        total_nodes += nodes;
        board.undo_move();
        println!("{}: {}", mov, nodes);
    }

    println!("Total: {}", total_nodes);
}

pub fn perft_bench(board: &mut Board, depth: u8) -> u64
{
    println!("Running perft depth {} on {}", depth, board.fen());
    let start = Instant::now();
    let nodes = perft(board, depth);

    println!("perft depth {} nodes {} nps {} time {} fen {}",
        depth, 
        nodes, 
        nodes * 1000 / milliseconds_elapsed(start).max(1) as u64,
        milliseconds_elapsed(start), 
        board.fen());

    nodes
}
