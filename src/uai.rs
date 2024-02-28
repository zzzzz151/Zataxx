use std::io;
use std::time::Instant;
use crate::types::*;
//use crate::utils::*;
use crate::ataxx_move::*;
use crate::board::*;
use crate::perft::*;
use crate::search::*;
use crate::bench::*;
use crate::datagen::*;

pub fn uai_loop(searcher: &mut Searcher)
{
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
                println!("option name Hash type spin default {} min 1 max 1024", DEFAULT_TT_SIZE_MB);
                println!("uaiok");
            }
            "setoption" => { 
                setoption(input_split, searcher);
            }
            "isready" => { 
                println!("readyok"); 
            }
            "uainewgame" => { 
                uainewgame(searcher);
            }
            "position" => { 
                position(input_split, searcher);
             }
            "go" => { 
                go(input_split, searcher);
             }
            "d" | "display" | "print" | "show" => { 
                searcher.board.print(); 
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
                let depth: u8 = input_split[1].parse::<u8>().unwrap();
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
}

pub fn position(tokens: Vec<&str>, searcher: &mut Searcher)
{
    // apply fen
    if tokens[1] == "startpos" {
       searcher.board = Board::new(START_FEN);
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
        searcher.board = Board::new(&fen);
    }

    // apply moves if any
    for (i, token) in tokens.iter().skip(2).enumerate()
    {
        if token == &"moves" {
            for j in (i+3)..tokens.len() {
                searcher.board.make_move(AtaxxMove::from_uai(tokens[j as usize]));
            }
            break;
        }
    }
}

pub fn go(tokens: Vec<&str>, searcher: &mut Searcher)
{
    searcher.start_time = Instant::now();
    searcher.milliseconds = U64_MAX;

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
            searcher.milliseconds = tokens[i+1].parse().unwrap();
            break;
        }
    }

    searcher.search(true);
    assert!(searcher.best_move_root != MOVE_NONE);
    println!("bestmove {}", searcher.best_move_root);
}