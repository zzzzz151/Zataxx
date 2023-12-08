use crate::board::*;
use crate::types::*;
use crate::utils::*;

pub fn perft(board: &mut Board, depth: u8) -> u64
{
    if depth == 0 {
        return 1u64
    }

    if board.get_game_result() != GameResult::None {
        return 0u64
    }

    let mut moves: MovesArray = EMPTY_MOVES_ARRAY;
    let num_moves = board.moves(&mut moves);
    let mut nodes: u64 = 0;

    for i in 0..num_moves
    {
        board.make_move(moves[i as usize]);
        nodes += perft(board, depth - 1);
        board.undo_move();
    }

    nodes
}

pub fn perft_split(board: &mut Board, depth: u8)
{
    assert!(depth > 0);

    let mut moves: MovesArray = EMPTY_MOVES_ARRAY;
    let num_moves = board.moves(&mut moves);
    let mut total_nodes: u64 = 0;

    for i in 0..num_moves
    {
        let mov: Move = moves[i as usize];
        board.make_move(mov);
        let nodes: u64 = perft(board, depth - 1);
        total_nodes += nodes;
        board.undo_move();
        println!("{}: {}", move_to_str(mov), nodes);
    }

    println!("Total: {}", total_nodes);
}
