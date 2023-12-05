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

    let mut moves: [Move; 256] = [MOVE_NONE; 256];
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

    let mut moves: [Move; 256] = [MOVE_NONE; 256];
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

const PERFT_TESTS: [(&str, [i32; 7]); 21] = [
    ("7/7/7/7/7/7/7 r 0 1", [1, 0, 0, 0, 0, -1, -1]),
    ("7/7/7/7/7/7/7 b 0 1", [1, 0, 0, 0, 0, -1, -1]),
    ("r5b/7/7/7/7/7/b5r r 100 1", [1, 0, 0, 0, 0, -1, -1]),
    ("r5b/7/7/7/7/7/b5r b 100 1", [1, 0, 0, 0, 0, -1, -1]),
    ("7/7/7/7/-------/-------/r5b r 0 1", [1, 2, 4, 13, 30, 73, 174]),
    ("7/7/7/7/-------/-------/r5b b 0 1", [1, 2, 4, 13, 30, 73, 174]),
    ("r5b/7/2-1-2/7/2-1-2/7/b5r r 0 1", [1, 14, 196, 4184, 86528, 2266352, -1]),
    ("r5b/7/2-1-2/7/2-1-2/7/b5r b 0 1", [1, 14, 196, 4184, 86528, 2266352, -1]),
    ("r5b/7/2-1-2/7/2-1-2/7/b5r r 0 1", [1, 14, 196, 4184, 86528, 2266352, -1]),
    ("r5b/7/2-1-2/3-3/2-1-2/7/b5r b 0 1", [1, 14, 196, 4100, 83104, 2114588, -1]),
    ("r5b/7/2-1-2/3-3/2-1-2/7/b5r r 0 1", [1, 14, 196, 4100, 83104, 2114588, -1]),
    ("r5b/7/3-3/2-1-2/3-3/7/b5r b 0 1", [1, 16, 256, 5948, 133264, 3639856, -1]),
    ("r5b/7/3-3/2-1-2/3-3/7/b5r r 0 1", [1, 16, 256, 5948, 133264, 3639856, -1]),
    ("r5b/7/7/7/7/7/b5r r 0 1", [1, 16, 256, 6460, 155888, 4752668, -1]),
    ("r5b/7/7/7/7/7/b5r b 0 1", [1, 16, 256, 6460, 155888, 4752668, -1]),
    ("7/7/7/2r1b2/7/7/7 r 0 1", [1, 23, 419, 7887, 168317, 4266992, -1]),
    ("7/7/7/2r1b2/7/7/7 b 0 1", [1, 23, 419, 7887, 168317, 4266992, -1]),
    ("7/7/7/7/bbbbbbb/bbbbbbb/rrrrrrr r 0 1", [1, 1, 75, 249, 14270, 452980, -1]),
    ("7/7/7/7/rrrrrrr/rrrrrrr/bbbbbbb b 0 1", [1, 1, 75, 249, 14270, 452980, -1]),
    ("7/7/7/7/bbbbbbb/bbbbbbb/rrrrrrr b 0 1", [1, 75, 249, 14270, 452980, -1, -1]),
    ("7/7/7/7/rrrrrrr/rrrrrrr/bbbbbbb r 0 1", [1, 75, 249, 14270, 452980, -1, -1]),
];


pub fn run_perft_tests()
{
    println!("Running perft tests...");

    for test_entry in &PERFT_TESTS 
    {
        let (fen, nodes_per_depth) = test_entry;
        let mut board: Board = Board::new(fen);

        for depth in 0..7 {
            let expected_nodes: i32 = nodes_per_depth[depth];
            if expected_nodes == -1 {
                continue;
            }
            let our_nodes: u64 = perft(&mut board, depth as u8);
            assert_eq!(our_nodes as i32, expected_nodes, 
                       "[Perft test] Expected {} nodes but got {}, '{}', depth {}",
                       expected_nodes, our_nodes, fen, depth);
        }
    }

    println!("Passed perft tests!");
}