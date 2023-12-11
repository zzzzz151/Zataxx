use rand::Rng;
use std::time::Instant;
use std::fs::File;
use std::io::prelude::*;
use crate::types::*;
//use crate::utils::*;
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

        if skip || zobrist_hashes_written.contains(&(search_data.board.zobrist_hash)) { 
            continue;
        }     

        search_data.tt.reset();
        search_data.start_time = Instant::now();
        let score: i16 = search(&mut search_data, false).1;

        if score.abs() <= 100 {
            let line: String = search_data.board.fen() + "\n";
            let _ = file.write_all(line.as_bytes());
            zobrist_hashes_written.push(search_data.board.zobrist_hash);
            println!("Openings written: {}", zobrist_hashes_written.len());
        }

    }
}