#![allow(dead_code)]
#![allow(unused_variables)]

use std::io;
use crate::types::*;
use crate::utils::*;
use crate::board::*;
use crate::perft::*;
use crate::search::*;

pub fn uai_loop()
{
    let mut input = String::new();
    let mut board: Board = Board::new(START_FEN);

    loop
    {
        let _ = io::stdin().read_line(&mut input);
        input = input.trim().to_string();
        let input_split: Vec<&str> = input.split(' ').map(str::trim).collect();

        match input_split[0] {
            "uai" => {
                println!("id name Zataxx");
                println!("id author zzzzz");
                println!("uaiok");
            }
            "setoption" => { setoption(input_split); }
            "uainewgame" => { uainewgame(); }
            "position" => { position(input_split, &mut board); }
            "go" => { go(input_split, &mut board); }
            "d" | "display" | "print" | "show" => { 
                board.print(); 
            }
            "perfttests" => { run_perft_tests(); }
            "perft" => {  
                let depth: u8 = input_split[1].parse::<u8>().unwrap();
                let nodes: u64 = perft(&mut board, depth);
                println!("perft depth {} nodes {}", depth, nodes);
            }
            "perftsplit" => { 
                let depth: u8 = input_split[1].parse::<u8>().unwrap();
                perft_split(&mut board, depth);
             }
            _ => { }
        }

        input.clear();
    }
}

pub fn setoption(tokens: Vec<&str>)
{

}

pub fn uainewgame()
{

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
        for (i, token) in tokens.iter().skip(2).enumerate() {
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
                board.make_move(str_to_move(tokens[j as usize]));
            }
            break;
        }
    }

}

pub fn go(tokens: Vec<&str>, board: &mut Board)
{
    let best_move: Move = search(board);
    println!("bestmove {}", move_to_str(best_move));
}