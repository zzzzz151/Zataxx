use std::time::Instant;
use crate::board::*;
use crate::types::*;
use crate::utils::*;
use crate::ataxx_move::*;

fn perft(board: &mut Board, depth: u8) -> u64
{
    if depth == 0 {
        return 1u64;
    }

    if board.get_game_result() != GameResult::None {
        return 0u64;
    }

    if depth == 1 {
        let mut moves: MovesList = MovesList::default();
        board.moves(&mut moves);
        return moves.num_moves as u64;
    }

    // Generate moves
    let mut moves: MovesList = MovesList::default();
    board.moves(&mut moves);
    let mut nodes: u64 = 0;

    for i in 0..(moves.num_moves as usize)
    {
        board.make_move(moves[i]);
        nodes += perft(board, depth - 1);
        board.undo_move();
    }

    nodes
}

pub fn perft_split(fen: &str, depth: u8)
{
    assert!(depth > 0);
    println!("Running split perft depth {} on {}", depth, fen);
    let mut board = Board::new(fen);
    board.nnue = false;

    // Generate moves
    let mut moves: MovesList = MovesList::default();
    board.moves(&mut moves);
    let mut total_nodes: u64 = 0;

    for i in 0..(moves.num_moves as usize)
    {
        let mov: AtaxxMove = moves[i];
        board.make_move(mov);
        let nodes: u64 = perft(&mut board, depth - 1);
        total_nodes += nodes;
        board.undo_move();
        println!("{}: {}", mov, nodes);
    }

    println!("Total: {}", total_nodes);
}

pub fn perft_bench(fen: &str, depth: u8) -> u64
{
    println!("Running perft depth {} on {}", depth, fen);
    let mut board = Board::new(fen);
    board.nnue = false;

    let start = Instant::now();
    let nodes = perft(&mut board, depth);

    println!("perft depth {} nodes {} nps {} time {} fen {}",
             depth, 
             nodes, 
             nodes * 1000 / milliseconds_elapsed(start).max(1) as u64,
             milliseconds_elapsed(start), 
             fen);

    nodes
}
