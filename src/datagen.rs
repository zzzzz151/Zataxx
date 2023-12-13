use rand::Rng;
use std::time::Instant;
use std::fs::File;
use std::io::prelude::*;
use crate::types::*;
use crate::utils::*;
use crate::tables::*;
use crate::board::*;
use crate::tt::*;
use crate::search::*;

pub fn generate_openings(file_path: &str, ply: u8, num_openings: u16)
{
    // Open the file in write mode, creating it if it doesn't exist
    let mut file = match File::create(&file_path) {
        Ok(file) => file,
        Err(e) => panic!("Error creating file {}: {}", file_path, e),
    };

    let mut zobrist_hashes_written: Vec<u64> = Vec::with_capacity(num_openings.into());
    let mut rng = rand::thread_rng();

    let mut search_data = SearchData {
        board: Board::default(),
        max_depth: 6,
        start_time: Instant::now(),
        milliseconds: 4294967295,
        turn_milliseconds: 0,
        time_is_up: false,
        soft_nodes: 4294967295,
        hard_nodes: 4294967295,
        best_move_root: MOVE_NONE,
        nodes: 0,
        tt: TT::new(DEFAULT_TT_SIZE_MB),
        lmr_table: get_lmr_table()
    };

    while zobrist_hashes_written.len() < num_openings.into()    
    {
        let fen: &str = if zobrist_hashes_written.len() % 2 == 0 { START_FEN } else { START_FEN2 };
        search_data.board = Board::new(fen);

        let mut skip = false;
        for _i in 0..ply {
            let mut moves: MovesArray = EMPTY_MOVES_ARRAY;
            let num_moves = search_data.board.moves(&mut moves);
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
        search_data.start_time = Instant::now();
        let score: i16 = search(&mut search_data, false).1;

        if score.abs() <= 100 {
            let line: String = search_data.board.fen() + "\n";
            let _ = file.write_all(line.as_bytes());
            zobrist_hashes_written.push(search_data.board.state.zobrist_hash);
            println!("Openings written: {}", zobrist_hashes_written.len());
        }

    }
}

#[allow(unused_assignments)]

pub fn datagen()
{    
    // random file name
    let characters = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let file_name: String = (0..12)
    .map(|_| {
        let random_index = rand::thread_rng().gen_range(0..characters.len());
        characters.chars().nth(random_index).unwrap()
    })
    .collect();

    let file_path = format!("data/{}.txt", file_name);

    // Open the file in write mode, creating it if it doesn't exist
    let mut file = match File::create(file_path.clone()) {
        Ok(file) => file,
        Err(e) => panic!("Error creating file {}: {}", file_path, e),
    };

    let mut search_data = SearchData {
        board: Board::default(),
        max_depth: 100,
        start_time: Instant::now(),
        milliseconds: 4294967295,
        turn_milliseconds: 0,
        time_is_up: false,
        soft_nodes: 5000,
        hard_nodes: 8_000_000,
        best_move_root: MOVE_NONE,
        nodes: 0,
        tt: TT::new(DEFAULT_TT_SIZE_MB),
        lmr_table: get_lmr_table()
    };

    let mut rng = rand::thread_rng();
    let mut positions_written: u32 = 0;
    let datagen_start_time = Instant::now();

    // Infinite loop
    loop 
    {
        // Apply random opening
        search_data.board = Board::new(if rng.gen_range(0..=1) == 0 {START_FEN} else {START_FEN2});
        let plies: u8 = rng.gen_range(11..=16) as u8;
        let mut plies_made = 0;
        loop {
            let mut moves: MovesArray = EMPTY_MOVES_ARRAY;
            let num_moves = search_data.board.moves(&mut moves);
            let random_index = rng.gen_range(0..num_moves);
            search_data.board.make_move(moves[random_index as usize]);

            if search_data.board.get_game_result() != GameResult::None || moves[0] == MOVE_PASS
            {
                while search_data.board.states.len() > 0 {
                    search_data.board.undo_move();
                }
                plies_made = 0;
                continue;
            }

            plies_made += 1;
            if plies_made == plies { break; }
        }

        search_data.tt.reset();
        let mut lines: Vec<String> = Vec::with_capacity(128);
        let mut game_result = GameResult::None;

        // Play out game
        loop {
            search_data.start_time = Instant::now();
            let (mov, mut score) = search(&mut search_data, false);
            assert!(mov != MOVE_NONE);

            if score >= 2000 {
                game_result = if search_data.board.state.color == Color::Red 
                                {GameResult::WinRed} 
                              else 
                                {GameResult::WinBlue};
                break;
            }
            else if score <= -2000 {
                game_result = if search_data.board.state.color == Color::Red 
                                {GameResult::WinBlue} 
                              else 
                                {GameResult::WinRed};
                break;
            }
            
            if search_data.board.state.color == Color::Blue {
                score = -score;
            }
            lines.push(format!("{} | {}", search_data.board.fen(), score));

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

        positions_written += lines.len() as u32;
        println!("{} | Positions: {} | Positions/sec: {}",
                 file_path, 
                 positions_written, 
                 positions_written * 1000 / milliseconds_elapsed(datagen_start_time));
    }

}

