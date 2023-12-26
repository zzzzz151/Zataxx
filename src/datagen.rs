#![allow(unused_assignments)]

use rand::Rng;
use std::time::Instant;
use std::fs::File;
use std::io::prelude::*;
use std::fs;
use crate::types::*;
use crate::utils::*;
use crate::board::*;
use crate::search::*;

pub fn datagen()
{    
    const SOFT_NODES: u64 = 5000;
    const HARD_NODES: u64 = 10_000_000;
    const OPENING_SCORE_THRESHOLD: i16 = 25;
    const ADJ_SCORE: i16 = 200;

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
    let start_board_4_blockers: Board = Board::new(START_FEN_4_BLOCKERS);
    let mut map: u8 = 0; // map=0 => no blockers, map=1 => 4 blockers
    let mut search_data = SearchData::new(Board::default(), 100, U64_MAX, SOFT_NODES, HARD_NODES);

    let mut rng = rand::thread_rng();
    let mut positions_written: u64 = 0;
    let datagen_start_time = Instant::now();

    // Infinite loop
    loop 
    {
        search_data.start_time = Instant::now();

        // Set start pos
        search_data.board = if map == 0 {start_board.clone()} else {start_board_4_blockers.clone()};
        map = 1 - map;

        // Apply random opening
        let plies: u8 = rng.gen_range(8..=15) as u8;
        loop {
            let mut moves: MovesArray = EMPTY_MOVES_ARRAY;
            let num_moves = search_data.board.moves(&mut moves);
            assert!(num_moves > 0);
            let random_index = rng.gen_range(0..num_moves);
            search_data.board.make_move(moves[random_index as usize]);

            if search_data.board.get_game_result() != GameResult::None || moves[0] == MOVE_PASS
            {
                search_data.board = if map == 0 {start_board.clone()} else {start_board_4_blockers.clone()};
                continue;
            }

            if search_data.board.states.len() == plies.into() 
            { 
                // Skip very unbalanced openings
                search_data.tt.reset();
                search_data.killers = [MOVE_NONE; 256];
                search_data.soft_nodes = SOFT_NODES * 2;
                let score = search(&mut search_data, false).1;
                search_data.soft_nodes = SOFT_NODES;
                if score.abs() >= OPENING_SCORE_THRESHOLD { 
                    search_data.board = if map == 0 {start_board.clone()} else {start_board_4_blockers.clone()};
                    continue; 
                }
                break;
            }
        }

        search_data.tt.reset();
        search_data.start_time = Instant::now();
        let mut lines: Vec<String> = Vec::with_capacity(128);
        let mut game_result = GameResult::None;

        // Play out game
        loop {
            search_data.killers = [MOVE_NONE; 256];
            let (mov, score) = search(&mut search_data, false);
            assert!(mov != MOVE_NONE);

            if score.abs() >= ADJ_SCORE {
                game_result = if (search_data.board.state.color == Color::Red && score > 0)
                              || (search_data.board.state.color == Color::Blue && score < 0)
                                  {GameResult::WinRed} 
                              else
                                  {GameResult::WinBlue};
                break;
            }

            lines.push(format!("{} | {}", 
                       search_data.board.fen(), 
                       if search_data.board.state.color == Color::Red {score} else {-score}));

            search_data.board.make_move(mov);
            game_result = search_data.board.get_game_result();
            if game_result != GameResult::None {
                break;
            }
        }

        assert!(game_result != GameResult::None);

        if search_data.board.state.plies_since_single >= 100 {
            continue;
        }

        // Write data from this game to file
        for i in 0..lines.len() {
            let line = format!("{} | {}\n", lines[i], game_result.to_string());
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
    let start_board_4_blockers: Board = Board::new(START_FEN_4_BLOCKERS);
    let mut search_data = SearchData::new(Board::default(), 100, U64_MAX, 1_000_000, 100_000_000);
    let mut zobrist_hashes_written: Vec<u64> = Vec::with_capacity(1024);
    let mut rng = rand::thread_rng();
    let ply: usize = 8;

    loop    
    {
        search_data.board = if zobrist_hashes_written.len() % 2 == 0 { start_board.clone() }
                            else { start_board_4_blockers.clone() };

        let mut skip = false;
        for _i in 0..ply {
            let mut moves: MovesArray = EMPTY_MOVES_ARRAY;
            let num_moves = search_data.board.moves(&mut moves);
            assert!(num_moves > 0);
            let random_index = rng.gen_range(0..num_moves) as usize;
            search_data.board.make_move(moves[random_index]);

            if search_data.board.get_game_result() != GameResult::None 
            || search_data.board.must_pass() {
                skip = true;
                break;
            }
        }

        if skip || zobrist_hashes_written.contains(&(search_data.board.state.zobrist_hash)) { 
            continue;
        }     

        search_data.tt.reset();
        search_data.killers = [MOVE_NONE; 256];
        search_data.start_time = Instant::now();
        let score: i16 = search(&mut search_data, false).1;

        if score.abs() <= 1 {
            let line: String = search_data.board.fen() + "\n";
            //print!("Writing opening {}", line);
            let _ = file.write_all(line.as_bytes());
            zobrist_hashes_written.push(search_data.board.state.zobrist_hash);
            println!("{} | Openings written: {}", file_path, zobrist_hashes_written.len());
        }
    }
}

