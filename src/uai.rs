use std::io;
use std::time::Instant;
use crate::types::*;
//use crate::utils::*;
use crate::ataxx_move::*;
use crate::board::*;
use crate::perft::*;
use crate::tt::*;
use crate::search::*;
use crate::bench::*;
use crate::datagen::*;

pub fn uai_loop(search_data: &mut SearchData)
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
                println!("uaiok");
            }
            "setoption" => { 
                setoption(input_split, search_data);
            }
            "isready" => { 
                println!("readyok"); 
            }
            "uainewgame" => { 
                uainewgame(search_data);
            }
            "position" => { 
                position(input_split, search_data);
             }
            "go" => { 
                go(input_split, search_data);
             }
            "d" | "display" | "print" | "show" => { 
                search_data.board.print(); 
            }
            "eval" | "evaluate" | "evaluation" => {
                println!("eval {}", search_data.board.evaluate());
            }
            "perft" => {  
                let depth: u8 = input_split[1].parse::<u8>().unwrap();
                perft_bench(&search_data.board.fen(), depth);
            }
            "perftsplit" | "splitperft" => { 
                let depth: u8 = input_split[1].parse::<u8>().unwrap();
                perft_split(&search_data.board.fen(), depth);
            }
            "bench" => {
                let depth: u8 = input_split[1].parse::<u8>().unwrap();
                bench(depth);
            }
            "gameresult" => {
                println!("{}", search_data.board.get_game_result().to_string());
            }
            "makemove" => {
                search_data.board.make_move(AtaxxMove::from_uai(input_split[1]));
            }
            "undomove" => {
                search_data.board.undo_move();
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

pub fn setoption(tokens: Vec<&str>, search_data: &mut SearchData)
{
    let option_name = tokens[2];
    let option_value = tokens[4];

    if option_name == "hash" || option_name == "Hash" {
        search_data.tt = TT::new(option_value.parse::<usize>().unwrap());
    }
}

pub fn uainewgame(search_data: &mut SearchData)
{
    search_data.tt.reset();
    search_data.killers = [MOVE_NONE; 256];
}

pub fn position(tokens: Vec<&str>, search_data: &mut SearchData)
{
    // apply fen
    if tokens[1] == "startpos" {
       search_data.board = Board::new(START_FEN);
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
        search_data.board = Board::new(&fen);
    }

    // apply moves if any
    for (i, token) in tokens.iter().skip(2).enumerate()
    {
        if token == &"moves" {
            for j in (i+3)..tokens.len() {
                search_data.board.make_move(AtaxxMove::from_uai(tokens[j as usize]));
            }
            break;
        }
    }
}

pub fn go(tokens: Vec<&str>, search_data: &mut SearchData)
{
    search_data.start_time = Instant::now();
    search_data.milliseconds = U64_MAX;

    let mut is_wtime_btime: bool = true;
    for i in (1..tokens.len()).step_by(2) {
        if tokens[i] == "rtime"
        {
            is_wtime_btime = false;
            break;
        }
    }

    for i in (1..tokens.len()).step_by(2) {
        if (tokens[i] == "rtime" && search_data.board.state.color == Color::Red)
        || (tokens[i] == "wtime" && search_data.board.state.color == Color::Blue)
        || (tokens[i] == "btime" 
        && ((is_wtime_btime && search_data.board.state.color == Color::Red)
        || (!is_wtime_btime && search_data.board.state.color == Color::Blue)))
        {
            search_data.milliseconds = tokens[i+1].parse().unwrap();
            break;
        }
    }

    search(search_data, true);
    assert!(search_data.best_move_root != MOVE_NONE);
    println!("bestmove {}", search_data.best_move_root);
}