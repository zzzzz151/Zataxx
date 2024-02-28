#![allow(unused_assignments)]

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

pub fn datagen()
{    
    // Create 'data' folder
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
    searcher.soft_nodes = 5000;
    searcher.hard_nodes = 1_000_000;

    let mut rng = rand::thread_rng();
    let mut positions_written: u64 = 0;
    let datagen_start_time = Instant::now();

    // Infinite loop
    loop {
        searcher.start_time = Instant::now();
        searcher.board = start_board.clone();
        let target_plies: u8 = rng.gen_range(16..=21) as u8;
        let mut moves: MovesList = MovesList::default();

        loop {
            // Generate moves and make a random one
            searcher.board.moves(&mut moves);
            let random_index = rng.gen_range(0..moves.size());
            searcher.board.make_move(moves[random_index as usize]);

            if moves[0] == MOVE_PASS || searcher.board.game_state().0 != GameState::Ongoing
            {
                searcher.board = start_board.clone();
                continue;
            }

            if searcher.board.states.len() == target_plies.into() 
            { 
                // Skip very unbalanced openings
                uainewgame(&mut searcher);
                let score = searcher.search(false).1;
                if score.abs() >= 200 { 
                    searcher.board = start_board.clone();
                    continue;
                }
                break;
            }
        }

        searcher.clear_tt();
        let mut lines: Vec<String> = Vec::with_capacity(128);
        let mut game_state = GameState::Ongoing;
        let mut winner = Color::None;

        // Play out game
        loop {
            searcher.clear_killers();
            let (mov, score) = searcher.search(false);
            assert!(mov != MOVE_NONE);

            // Adjudication
            if score.abs() >= 2500 {
                game_state = GameState::Won;
                winner = if score > 0 { 
                    searcher.board.state.color 
                } else {
                    opp_color(searcher.board.state.color)
                };

                break;
            }

            lines.push(format!("{} | {}", 
                searcher.board.fen(), 
                if searcher.board.state.color == Color::Red {score} else {-score}));

            searcher.board.make_move(mov);
            (game_state, winner) = searcher.board.game_state();
            if game_state != GameState::Ongoing {
                break;
            }
        }

        assert!(game_state != GameState::Ongoing);
        if game_state != GameState::Draw {
            assert!(winner != Color::None);
        }

        if searcher.board.state.plies_since_single >= 100 {
            continue;
        }

        // Write data from this game to file
        for i in 0..lines.len() 
        {
            let line = format!("{} | {}\n", 
                lines[i], 
                if winner == Color::Red {
                    "1.0"
                } else if winner == Color::Blue {
                    "0.0"
                } else {
                    "0.5"
                });

            let _ = file.write_all(line.as_bytes());
        }

        positions_written += lines.len() as u64;
        println!("{} | Positions: {} | Positions/sec: {}",
            file_path, 
            positions_written, 
            positions_written * 1000 / milliseconds_elapsed(datagen_start_time));
    }

}

pub const CHARACTERS: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

pub fn datagen_openings()
{
    // Create 'data' folder
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
    searcher.soft_nodes = 1_000_000;
    searcher.hard_nodes = 100_000_000;

    let mut zobrist_hashes_written: Vec<u64> = Vec::with_capacity(1024);
    let mut rng = rand::thread_rng();
    let target_ply: usize = 8;
    let mut moves: MovesList = MovesList::default();

    loop {
        searcher.board = start_board.clone();

        // Get a random opening with target_ply plies
        loop {
            // Generate moves and make a random one
            searcher.board.moves(&mut moves);
            let random_index = rng.gen_range(0..moves.size()) as usize;
            searcher.board.make_move(moves[random_index]);  

            if searcher.board.game_state().0 != GameState::Ongoing
            || searcher.board.must_pass() {
                searcher.board = start_board.clone();
                continue;
            }

            if searcher.board.states.len() == target_ply {
                break;
            }
        }

        assert!(searcher.board.game_state().0 == GameState::Ongoing);
        assert!(!searcher.board.must_pass());

        if zobrist_hashes_written.contains(&(searcher.board.state.zobrist_hash)) { 
            continue;
        }     

        uainewgame(&mut searcher);
        searcher.start_time = Instant::now();
        let score = searcher.search(false).1;

        if score.abs() <= 1 {
            let line: String = searcher.board.fen() + "\n";
            //print!("Writing opening {}", line);
            let _ = file.write_all(line.as_bytes());
            zobrist_hashes_written.push(searcher.board.state.zobrist_hash);
            println!("{} | Openings written: {}", file_path, zobrist_hashes_written.len());
        }
    }
}

