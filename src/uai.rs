use std::io;
use std::time::Instant;
use crate::types::*;
use crate::utils::*;
use crate::nnue::*;
use crate::board::*;
use crate::perft::*;
use crate::tt::*;
use crate::search::*;
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
                println!("eval {} cp", 
                         evaluate(search_data.board.state.color, &search_data.board.state.accumulator));
            }
            "perft" => {  
                let depth: u8 = input_split[1].parse::<u8>().unwrap();
                let nodes: u64 = perft(&mut search_data.board, depth);
                println!("perft depth {} nodes {}", depth, nodes);
            }
            "perftsplit" => { 
                let depth: u8 = input_split[1].parse::<u8>().unwrap();
                perft_split(&mut search_data.board, depth);
            }
            "gameresult" => {
                println!("{}", search_data.board.get_game_result().to_string());
            }
            "genopenings" => {
                generate_openings("openings.txt", 4, 3000);
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
                search_data.board.make_move(str_to_move(tokens[j as usize]));
            }
            break;
        }
    }
}

pub fn go(tokens: Vec<&str>, search_data: &mut SearchData)
{
    search_data.start_time = Instant::now();
    search_data.milliseconds = 4294967295;
    for i in 1..tokens.len() {
        if (tokens[i] == "rtime" && search_data.board.state.color == Color::Red)
        || (tokens[i] == "btime" && search_data.board.state.color == Color::Blue)
        {
            search_data.milliseconds = tokens[i+1].parse().unwrap();
        }
    }

    let best_move: Move = search(search_data, true).0;
    assert!(best_move != MOVE_NONE);
    println!("bestmove {}", move_to_str(best_move));
}