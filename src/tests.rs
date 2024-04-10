#[cfg(test)]
mod tests {
    use crate::types::*;
    use crate::ataxx_move::*;
    use crate::board::*;
    use crate::perft::*;

    #[test]
    fn test_move_equality() {
        let mov1 = AtaxxMove::double(3, 5);
        let mov2 = AtaxxMove::double(3, 5);
        let mov3 = AtaxxMove::double(3, 6);
        assert!(mov1 == mov2);
        assert!(mov1 != mov3);
    }
    
    #[test]
    fn test_game_over() {
        let game_over_tests: [(&str, bool); 21] = [
            ("x5o/7/7/7/7/7/o5x x 0 1", false),
            ("x5o/7/2-1-2/7/2-1-2/7/o5x x 0 1", false),
            ("x5o/7/3-3/2-1-2/3-3/7/o5x x 0 1", false),
            ("7/7/7/7/7/7/7 x 0 1", true),
            ("7/7/7/7/7/7/7 o 0 1", true),
            ("x5o/7/7/7/7/7/o5x x 0 1", false),
            ("x5o/7/7/7/7/7/o5x x 99 1", false),
            ("x5o/7/7/7/7/7/o5x x 100 1", true),
            ("x5o/7/7/7/7/7/o5x x 101 1", true),
            ("x6/7/7/7/7/7/7 x 0 1", true),
            ("x6/7/7/7/7/7/7 o 0 1", true),
            ("o6/7/7/7/7/7/7 x 0 1", true),
            ("o6/7/7/7/7/7/7 o 0 1", true),
            ("7/7/7/7/4ooo/4ooo/4oox x 0 1", false),
            ("7/7/7/7/4ooo/4ooo/4oox o 0 1", false),
            ("7/7/7/7/-------/-------/x5o x 0 1", false),
            ("7/7/7/7/-------/-------/x5o o 0 1", false),
            ("7/7/7/7/-------/-------/xxxoooo x 0 1", true),
            ("7/7/7/7/-------/-------/xxxoooo o 0 1", true),
            ("7/7/7/7/---1---/-------/xxxoooo x 0 1", false),
            ("7/7/7/7/---1---/-------/xxxoooo o 0 1", false),
        ];

        for test in game_over_tests.iter() 
        {
            let mut board: Board = Board::new(test.0);
            if test.1 {
                assert!(board.game_state().0 != GameState::Ongoing);
            }
            else {
                assert!(board.game_state().0 == GameState::Ongoing);
            }
        }
    }

    #[test]
    fn test_winner()
    {
        let game_winner_tests: Vec<(&str, Color)> = vec![
            ("x5o/7/7/7/7/7/o5x x 0 1", Color::None),
            ("x5o/7/7/7/7/7/o5x o 0 1", Color::None),
            ("x5o/7/2-1-2/7/2-1-2/7/o5x x 0 1", Color::None),
            ("x5o/7/2-1-2/7/2-1-2/7/o5x o 0 1", Color::None),
            ("x6/7/7/7/7/7/7 x 0 1", Color::Red),
            ("x6/7/7/7/7/7/7 o 0 1", Color::Red),
            ("o6/7/7/7/7/7/7 x 0 1", Color::Blue),
            ("o6/7/7/7/7/7/7 o 0 1", Color::Blue),
            ("1xxxxxx/xxxxxxx/xxxxxxx/xxxxooo/ooooooo/ooooooo/ooooooo x 0 1", Color::None),
            ("1xxxxxx/xxxxxxx/xxxxxxx/xxxxooo/ooooooo/ooooooo/ooooooo o 0 1", Color::None),
            ("1oooooo/ooooooo/ooooooo/ooooxxx/xxxxxxx/xxxxxxx/xxxxxxx x 0 1", Color::None),
            ("1oooooo/ooooooo/ooooooo/ooooxxx/xxxxxxx/xxxxxxx/xxxxxxx o 0 1", Color::None),
            ("xxxxxxx/xxxxxxx/xxxxxxx/xxxxooo/ooooooo/ooooooo/ooooooo x 0 1", Color::Red),
            ("xxxxxxx/xxxxxxx/xxxxxxx/xxxxooo/ooooooo/ooooooo/ooooooo o 0 1", Color::Red),
            ("ooooooo/ooooooo/ooooooo/ooooxxx/xxxxxxx/xxxxxxx/xxxxxxx x 0 1", Color::Blue),
            ("ooooooo/ooooooo/ooooooo/ooooxxx/xxxxxxx/xxxxxxx/xxxxxxx o 0 1", Color::Blue),
            ("x5o/7/7/7/7/7/o5x x 99 0", Color::None),
            ("x5o/7/7/7/7/7/o5x x 100 0", Color::None),
            ("x6/7/7/7/7/7/7 x 100 0", Color::Red),
            ("x6/7/7/7/7/7/7 o 100 0", Color::Red),
            ("o6/7/7/7/7/7/7 x 100 0", Color::Blue),
            ("o6/7/7/7/7/7/7 o 100 0", Color::Blue),
            ("x5o/7/7/7/7/7/o5x x 0 400", Color::None),
        ];
    
        for test in game_winner_tests.iter() {
            let mut board: Board = Board::new(test.0);
            assert_eq!(board.game_state().1, test.1);
        }
    }

    #[test]
    fn test_perft() {
        let perft_tests: [(&str, Vec<u64>); 21] = [
            ("7/7/7/7/7/7/7 r 0 1", vec![1, 0, 0]),
            ("7/7/7/7/7/7/7 b 0 1", vec![1, 0, 0]),
            ("r5b/7/7/7/7/7/b5r r 100 1", vec![1, 0, 0]),
            ("r5b/7/7/7/7/7/b5r b 100 1", vec![1, 0, 0]),
            ("7/7/7/7/-------/-------/r5b r 0 1", vec![1, 2, 4, 13, 30, 73, 174]),
            ("7/7/7/7/-------/-------/r5b b 0 1", vec![1, 2, 4, 13, 30, 73, 174]),
            ("r5b/7/2-1-2/7/2-1-2/7/b5r r 0 1", vec![1, 14, 196, 4184, 86528, 2266352]),
            ("r5b/7/2-1-2/7/2-1-2/7/b5r b 0 1", vec![1, 14, 196, 4184, 86528, 2266352]),
            ("r5b/7/2-1-2/7/2-1-2/7/b5r r 0 1", vec![1, 14, 196, 4184, 86528, 2266352]),
            ("r5b/7/2-1-2/3-3/2-1-2/7/b5r b 0 1", vec![1, 14, 196, 4100, 83104, 2114588]),
            ("r5b/7/2-1-2/3-3/2-1-2/7/b5r r 0 1", vec![1, 14, 196, 4100, 83104, 2114588]),
            ("r5b/7/3-3/2-1-2/3-3/7/b5r b 0 1", vec![1, 16, 256, 5948, 133264, 3639856]),
            ("r5b/7/3-3/2-1-2/3-3/7/b5r r 0 1", vec![1, 16, 256, 5948, 133264, 3639856]),
            ("r5b/7/7/7/7/7/b5r r 0 1", vec![1, 16, 256, 6460, 155888, 4752668]),
            ("r5b/7/7/7/7/7/b5r b 0 1", vec![1, 16, 256, 6460, 155888, 4752668]),
            ("7/7/7/2r1b2/7/7/7 r 0 1", vec![1, 23, 419, 7887, 168317, 4266992]),
            ("7/7/7/2r1b2/7/7/7 b 0 1", vec![1, 23, 419, 7887, 168317, 4266992]),
            ("7/7/7/7/bbbbbbb/bbbbbbb/rrrrrrr r 0 1", vec![1, 1, 75, 249, 14270, 452980]),
            ("7/7/7/7/rrrrrrr/rrrrrrr/bbbbbbb b 0 1", vec![1, 1, 75, 249, 14270, 452980]),
            ("7/7/7/7/bbbbbbb/bbbbbbb/rrrrrrr b 0 1", vec![1, 75, 249, 14270, 452980]),
            ("7/7/7/7/rrrrrrr/rrrrrrr/bbbbbbb r 0 1", vec![1, 75, 249, 14270, 452980]),
        ];

        for test_entry in &perft_tests {
            let (fen, nodes_by_depth) = test_entry;
            let mut board: Board = Board::new(fen);

            for depth in 1..nodes_by_depth.len()
            {
                let expected_nodes = nodes_by_depth[depth];
                assert_eq!(perft(&mut board, depth as u8), expected_nodes);
            }
        }
    }

    #[test]
    fn test_fen_hash_make_undo_move()
    {
        let mut board = Board::new(START_FEN);
        let fen = board.fen();
        let hash = board.zobrist_hash();
        assert_eq!(fen, START_FEN);

        board.make_move(AtaxxMove::from_uai("b6"));
        board.undo_move();
        assert_eq!(board.fen(), fen);
        assert_eq!(board.zobrist_hash(), hash);

        board.make_move(MOVE_PASS);
        board.undo_move();
        assert_eq!(board.fen(), fen);
        assert_eq!(board.zobrist_hash(), hash);
    }

}
