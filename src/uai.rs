use std::io;
use crate::types::*;
use crate::ataxx_move::*;
use crate::board::*;
use crate::perft::*;
use crate::search::*;
use crate::bench::*;
use crate::datagen::*;

pub fn uai_loop()
{
    let mut searcher: Searcher = Searcher::new(Board::new(START_FEN));

    loop
    {
        let mut input = String::new();
        let _ = io::stdin().read_line(&mut input);
        input = input.trim().to_string();
        let input_split: Vec<&str> = input.split(' ').map(str::trim).collect();

        match input_split[0] {
            "quit" => {
                break;
            }
            "uai" => {
                println!("id name Zataxx");
                println!("id author zzzzz");
                println!("option name Hash type spin default {} min 1 max 1024", TT_DEFAULT_MB);
                println!("uaiok");
            }
            "setoption" => { 
                setoption(input_split, &mut searcher);
            }
            "isready" => { 
                println!("readyok"); 
            }
            "uainewgame" => { 
                uainewgame(&mut searcher);
            }
            "position" => { 
                position(input_split, &mut searcher.board);
             }
            "go" => { 
                go(input_split, &mut searcher);
             }
             "d" | "display" | "print" | "show" => 
             {
                uainewgame(&mut searcher);

                let score = searcher.search(DEFAULT_MAX_DEPTH, I64_MAX, 0, 
                                true, 100_000, 1_000_000, false).1 as i64;

                uainewgame(&mut searcher);

                println!("+-------+-------+-------+-------+-------+-------+-------+");

                for row_idx in (0..=6).rev() {
                    println!("|       |       |       |       |       |       |       |");

                    print!("|");
                    for col_idx in 0..=6 
                    {
                        let sq: Square = (row_idx * 7 + col_idx) as Square;
                        let piece: char = searcher.board.piece_at(sq);
                        print!("   {}   |", piece);    
                    }
                    println!(" {}", row_idx + 1);

                    print!("|");
                    for col_idx in 0..=6 
                    {
                        let sq: Square = (row_idx * 7 + col_idx) as Square;
                        let piece_color: Color = searcher.board.color_at(sq);
                        if piece_color == Color::None {
                            print!("       |");
                        }
                        else {
                            searcher.board.remove_piece(piece_color, sq);

                            let score_no_piece = searcher.search(DEFAULT_MAX_DEPTH, I64_MAX, 0, 
                                                     true, 100_000, 1_000_000, false).1 as i64;

                            uainewgame(&mut searcher);

                            print!("{:^7}|", score - score_no_piece);

                            searcher.board.place_piece(piece_color, sq);
                        }
                    }
                    println!();

                    println!("+-------+-------+-------+-------+-------+-------+-------+");
                }

                println!("    A       B       C       D       E       F       G");
                println!();
                println!("Fen: {}", searcher.board.fen());
                println!("Zobrist hash: {}", searcher.board.state.zobrist_hash);
                println!("Eval: {} ", searcher.board.evaluate());
             }
            "eval" | "evaluate" | "evaluation" => {
                println!("eval {}", searcher.board.evaluate());
            }
            "perft" => {  
                let depth: u8 = input_split[1].parse::<u8>().unwrap();
                perft_bench(&searcher.board.fen(), depth);
            }
            "perftsplit" | "splitperft" => { 
                let depth: u8 = input_split[1].parse::<u8>().unwrap();
                perft_split(&searcher.board.fen(), depth);
            }
            "bench" => {
                let depth: u8 = if input_split.len() == 2 { 
                    input_split[1].parse::<u8>().unwrap() 
                }
                else {
                    DEFAULT_BENCH_DEPTH
                };

                bench(depth);
            }
            "makemove" => {
                searcher.board.make_move(AtaxxMove::from_uai(input_split[1]));
            }
            "undomove" => {
                searcher.board.undo_move();
            }
            "datagen_openings" => {
                datagen_openings();
            }
            "datagen" => {
                datagen();
            }
            _ => { }
        }
    }
}

pub fn setoption(tokens: Vec<&str>, searcher: &mut Searcher)
{
    let option_name = tokens[2];
    let option_value = tokens[4];

    if option_name == "hash" || option_name == "Hash" 
    {
        let size_mb: usize = option_value.parse().unwrap();
        searcher.resize_tt(size_mb);
    }
}

pub fn uainewgame(searcher: &mut Searcher)
{
    searcher.clear_tt();
    searcher.clear_killers();
    searcher.clear_history();
}

pub fn position(tokens: Vec<&str>, board: &mut Board)
{
    // apply fen
    if tokens[1] == "startpos" {
       *board = Board::new(START_FEN);
    }
    else if tokens[1] == "fen"
    {
        let mut fen = String::new();
        for token in tokens.iter().skip(2) {
            if token == &"moves" {
                break;
            }
            fen += token;
            fen.push(' ');
        }
        fen.pop(); // remove last whitespace
        *board = Board::new(&fen);
    }

    // apply moves if any
    for (i, token) in tokens.iter().skip(2).enumerate()
    {
        if token == &"moves" {
            for j in (i+3)..tokens.len() {
                board.make_move(AtaxxMove::from_uai(tokens[j as usize]));
            }
            break;
        }
    }
}

pub fn go(tokens: Vec<&str>, searcher: &mut Searcher)
{
    let mut milliseconds: i64 = I64_MAX;
    let mut increment_ms: u64 = 0;
    let mut is_move_time = false;
    let mut depth = DEFAULT_MAX_DEPTH;
    let mut nodes = U64_MAX;

    let mut is_wtime_btime: bool = true;
    for i in (1..tokens.len()).step_by(2) {
        if tokens[i] == "rtime" {
            is_wtime_btime = false;
            break;
        }
    }

    for i in (1..tokens.len()).step_by(2) 
    {
        if (tokens[i] == "rtime" && searcher.board.state.color == Color::Red)
        || (tokens[i] == "wtime" && searcher.board.state.color == Color::Blue)
        || (tokens[i] == "btime" 
        && ((is_wtime_btime && searcher.board.state.color == Color::Red)
        || (!is_wtime_btime && searcher.board.state.color == Color::Blue)))
        {
            milliseconds = tokens[i+1].parse().unwrap();
        }
        else if (tokens[i] == "rinc" && searcher.board.state.color == Color::Red)
        || (tokens[i] == "winc" && searcher.board.state.color == Color::Blue)
        || (tokens[i] == "binc" 
        && ((is_wtime_btime && searcher.board.state.color == Color::Red)
        || (!is_wtime_btime && searcher.board.state.color == Color::Blue)))
        {
            increment_ms = tokens[i+1].parse().unwrap();
        }
        else if tokens[i] == "movetime" {
            is_move_time = true;
            milliseconds = tokens[i+1].parse().unwrap();
        }
        else if tokens[i] == "depth" {
            depth = tokens[i+1].parse::<i64>().unwrap().clamp(0, 255) as u8;
        }
        else if tokens[i] == "nodes" {
            nodes = tokens[i+1].parse().unwrap();
        }
    }

    if is_move_time { increment_ms = 0; }

    let best_move = searcher.search(depth, milliseconds, increment_ms, 
                        is_move_time, U64_MAX, nodes, true).0;

    assert!(best_move != MOVE_NONE);
    println!("bestmove {}", best_move);
}