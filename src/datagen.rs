use rand::Rng;
use std::time::Instant;
use std::fs::File;
use std::io::prelude::*;
use std::fs;
use crate::uai::*;
use crate::types::*;
use crate::utils::*;
use crate::ataxx_move::*;
use crate::board::*;
use crate::search::*;

pub const CHARACTERS: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

pub fn datagen()
{   
    pub const MIN_PLIES: u8 = 14;
    pub const MAX_PLIES: u8 = 17;
    pub const SOFT_NODES: u64 = 5000;
    pub const HARD_NODES: u64 = 1_000_000;
    pub const MAX_OPENING_SCORE: i32 = 200;
    pub const ADJUDICATION_SCORE: i32 = 4000;

    // Create 'data' folder if doesnt exist
    let _ = fs::create_dir_all("data");

    // random file name
    let file_name: String = (0..12)
    .map(|_| {
        let random_index = rand::thread_rng().gen_range(0..CHARACTERS.len());
        CHARACTERS.chars().nth(random_index).unwrap()
    })
    .collect();

    let file_path = format!("data/{}.txt", file_name);

    // Open the file in write mode, creating it if it doesn't exist
    let mut file = match File::create(file_path.clone()) {
        Ok(file) => file,
        Err(e) => panic!("Error creating file {}: {}", file_path, e),
    };

    let start_board: Board = Board::new(START_FEN);
    let mut searcher = Searcher::new(start_board.clone());

    let mut rng = rand::thread_rng();
    let mut positions_written: u64 = 0;
    let datagen_start_time = Instant::now();

    // Infinite loop
    loop {
        searcher.board = start_board.clone();
        let num_random_plies: u8 = rng.gen_range(MIN_PLIES..=MAX_PLIES) as u8;
        let mut moves: MovesList = MovesList::default();

        // This loop gets a random opening
        loop {
            // Generate moves and make a random one
            searcher.board.moves(&mut moves);
            let random_index = rng.gen_range(0..moves.size());
            searcher.board.make_move(moves[random_index as usize]);

            // If pass move or game over, restart from start pos
            if moves[0] == MOVE_PASS || searcher.board.game_state().0 != GameState::Ongoing
            {
                searcher.board = start_board.clone();
                continue;
            }

            // If we made enough random moves
            if searcher.board.states.len() == num_random_plies.into() 
            { 
                // Skip very unbalanced openings

                uainewgame(&mut searcher);

                let score = searcher.search(DEFAULT_MAX_DEPTH, I64_MAX, 0, 
                                true, SOFT_NODES, HARD_NODES, false).1;

                if score.abs() > MAX_OPENING_SCORE { 
                    searcher.board = start_board.clone();
                    continue;
                }

                break;
            }
        }

        searcher.clear_tt();
        let mut lines: Vec<String> = Vec::with_capacity(128);
        let mut game_state: GameState;
        let mut winner: Color;

        // Play out game
        loop {
            searcher.clear_killers();
            let (mov, score) = searcher.search(DEFAULT_MAX_DEPTH, I64_MAX, 0, 
                                   true, SOFT_NODES, HARD_NODES, false);
            assert!(mov != MOVE_NONE);

            // Adjudication
            if score.abs() >= ADJUDICATION_SCORE {
                game_state = GameState::Won;
                winner = if score > 0 { 
                    searcher.board.state.color 
                } else {
                    opp_color(searcher.board.state.color)
                };

                break;
            }

            // <fen> | <move uai> | <score red/black pov>
            lines.push(format!("{} | {} | {}", 
                searcher.board.fen(), 
                mov,
                if searcher.board.state.color == Color::Red {score} else {-score}));

            searcher.board.make_move(mov);

            // if game over, break
            (game_state, winner) = searcher.board.game_state();
            if game_state != GameState::Ongoing {
                break;
            }
        }

        assert!(game_state != GameState::Ongoing);
        if game_state != GameState::Draw {
            assert!(winner != Color::None);
        }

        // Skip 100 ply draws since they are bad data
        if searcher.board.state.plies_since_single >= 100 {
            continue;
        }

        // Write data from this game to file
        for i in 0..lines.len() 
        {
            // Append "| <wdl red/black pov>" to the line
            let line = format!("{} | {}\n", 
                lines[i], 
                if winner == Color::Red {
                    "1.0"
                } else if winner == Color::Blue {
                    "0.0"
                } else {
                    "0.5"
                });

            // Write line to file
            let _ = file.write_all(line.as_bytes());
        }

        positions_written += lines.len() as u64;
        println!("{} | Positions: {} | Positions/sec: {}",
            file_path, 
            positions_written, 
            positions_written * 1000 / milliseconds_elapsed(datagen_start_time));
    }

}

pub fn datagen_openings()
{
    pub const PLY: usize = 8;
    pub const SOFT_NODES: u64 = 1_000_000;
    pub const HARD_NODES: u64 = 100_000_000;
    pub const MAX_OPENING_SCORE: i32 = 3;

    // Create 'data' folder if doesnt exist
    let _ = fs::create_dir_all("data");

    // random file name
    let file_name: String = (0..12)
    .map(|_| {
        let random_index = rand::thread_rng().gen_range(0..CHARACTERS.len());
        CHARACTERS.chars().nth(random_index).unwrap()
    })
    .collect();

    let file_path = format!("data/{}.txt", file_name);

    // Open the file in write mode, creating it if it doesn't exist
    let mut file = match File::create(file_path.clone()) {
        Ok(file) => file,
        Err(e) => panic!("Error creating file {}: {}", file_path, e),
    };

    let start_board: Board = Board::new(START_FEN);
    let mut searcher = Searcher::new(start_board.clone());

    let mut zobrist_hashes_written: Vec<u64> = Vec::with_capacity(1024);
    let mut rng = rand::thread_rng();
    let mut moves: MovesList = MovesList::default();

    // Inifnite loop
    loop {
        searcher.board = start_board.clone();

        // This loop gets a random opening with PLY plies
        loop {
            // Generate moves and make a random one
            searcher.board.moves(&mut moves);
            let random_index = rng.gen_range(0..moves.size()) as usize;
            searcher.board.make_move(moves[random_index]);  

            // If must pass or game over, restart from start pos
            if searcher.board.game_state().0 != GameState::Ongoing
            || searcher.board.must_pass() {
                searcher.board = start_board.clone();
                continue;
            }

            // If we made enough moves, break
            if searcher.board.states.len() == PLY {
                break;
            }
        }

        assert!(searcher.board.game_state().0 == GameState::Ongoing);
        assert!(!searcher.board.must_pass());

        // Skip opening if already found before
        if zobrist_hashes_written.contains(&(searcher.board.state.zobrist_hash)) { 
            continue;
        }     

        uainewgame(&mut searcher);
        let score = searcher.search(DEFAULT_MAX_DEPTH, I64_MAX, 0, 
                        true, SOFT_NODES, HARD_NODES, false).1;

        if score.abs() <= MAX_OPENING_SCORE 
        {
            // Write fen to file and save zobrist hash
            let line: String = searcher.board.fen() + "\n";
            let _ = file.write_all(line.as_bytes());
            zobrist_hashes_written.push(searcher.board.state.zobrist_hash);
            println!("{} | Openings written: {}", file_path, zobrist_hashes_written.len());
        }
    }
}

