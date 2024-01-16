#[cfg(test)]
mod tests {
    use crate::Board;
    use crate::GameResult;
    use crate::perft::*;
    use crate::ataxx_move::*;

    #[test]
    fn test_move() {
        let mov1 = AtaxxMove::double(3, 5);
        let mov2 = AtaxxMove::double(3, 5);
        let mov3 = AtaxxMove::double(3, 6);
        assert!(mov1 == mov2);
        assert!(mov1 != mov3);
    }
    
    #[test]
    fn test_board_game_over() {

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

        for test in game_over_tests.iter() {
            let mut board: Board = Board::new(test.0, true);
            if test.1 == false {
                assert!(board.get_game_result() == GameResult::None);
            }
            else {
                assert!(board.get_game_result() != GameResult::None);
            }
        }
    }

    #[test]
    fn test_board_result()
    {
        let tests: Vec<(&str, GameResult)> = vec![
            ("x5o/7/7/7/7/7/o5x x 0 1", GameResult::None),
            ("x5o/7/7/7/7/7/o5x o 0 1", GameResult::None),
            ("x5o/7/2-1-2/7/2-1-2/7/o5x x 0 1", GameResult::None),
            ("x5o/7/2-1-2/7/2-1-2/7/o5x o 0 1", GameResult::None),
            ("x6/7/7/7/7/7/7 x 0 1", GameResult::WinRed),
            ("x6/7/7/7/7/7/7 o 0 1", GameResult::WinRed),
            ("o6/7/7/7/7/7/7 x 0 1", GameResult::WinBlue),
            ("o6/7/7/7/7/7/7 o 0 1", GameResult::WinBlue),
            ("1xxxxxx/xxxxxxx/xxxxxxx/xxxxooo/ooooooo/ooooooo/ooooooo x 0 1", GameResult::None),
            ("1xxxxxx/xxxxxxx/xxxxxxx/xxxxooo/ooooooo/ooooooo/ooooooo o 0 1", GameResult::None),
            ("1oooooo/ooooooo/ooooooo/ooooxxx/xxxxxxx/xxxxxxx/xxxxxxx x 0 1", GameResult::None),
            ("1oooooo/ooooooo/ooooooo/ooooxxx/xxxxxxx/xxxxxxx/xxxxxxx o 0 1", GameResult::None),
            ("xxxxxxx/xxxxxxx/xxxxxxx/xxxxooo/ooooooo/ooooooo/ooooooo x 0 1", GameResult::WinRed),
            ("xxxxxxx/xxxxxxx/xxxxxxx/xxxxooo/ooooooo/ooooooo/ooooooo o 0 1", GameResult::WinRed),
            ("ooooooo/ooooooo/ooooooo/ooooxxx/xxxxxxx/xxxxxxx/xxxxxxx x 0 1", GameResult::WinBlue),
            ("ooooooo/ooooooo/ooooooo/ooooxxx/xxxxxxx/xxxxxxx/xxxxxxx o 0 1", GameResult::WinBlue),
            ("x5o/7/7/7/7/7/o5x x 99 0", GameResult::None),
            ("x5o/7/7/7/7/7/o5x x 100 0", GameResult::Draw),
            ("x6/7/7/7/7/7/7 x 100 0", GameResult::WinRed),
            ("x6/7/7/7/7/7/7 o 100 0", GameResult::WinRed),
            ("o6/7/7/7/7/7/7 x 100 0", GameResult::WinBlue),
            ("o6/7/7/7/7/7/7 o 100 0", GameResult::WinBlue),
            ("x5o/7/7/7/7/7/o5x x 0 400", GameResult::None),
        ];
    
        for test in tests.iter() {
            let mut board: Board = Board::new(test.0, true);
            assert_eq!(board.get_game_result(), test.1);
        }
    }

    #[test]
    fn test_perft()
    {
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

        for test_entry in &PERFT_TESTS 
        {
            let (fen, nodes_per_depth) = test_entry;

            for depth in 0..7 {
                let expected_nodes: i32 = nodes_per_depth[depth];
                if expected_nodes == -1 {
                    continue;
                }
                let our_nodes: u64 = perft_bench(fen, depth as u8);
                assert_eq!(our_nodes as i32, expected_nodes);
            }
    }


    }

}
