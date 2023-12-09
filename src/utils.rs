#![allow(dead_code)]
#![allow(unused_variables)]

use std::time::Instant;
use crate::types::*;

/*
42 43 44 45 46 47 48
35 36 37 38 39 40 41
28 29 30 31 32 33 34
21 22 23 24 25 26 27
14 15 16 17 18 19 20
 7  8  9 10 11 12 13
 0  1  2  3  4  5  6
*/

pub const SQUARE_TO_STR: [&str; 49] = [
    "a1", "b1", "c1", "d1", "e1", "f1", "g1",
    "a2", "b2", "c2", "d2", "e2", "f2", "g2", 
    "a3", "b3", "c3", "d3", "e3", "f3", "g3", 
    "a4", "b4", "c4", "d4", "e4", "f4", "g4",
    "a5", "b5", "c5", "d5", "e5", "f5", "g5",
    "a6", "b6", "c6", "d6", "e6", "f6", "g6", 
    "a7", "b7", "c7", "d7", "e7", "f7", "g7", 
];

pub fn square_rank(sq: Square) -> Rank {
    unsafe { std::mem::transmute((sq / 7) as u8) }
}

pub fn square_file(sq: Square) -> File {
    unsafe { std::mem::transmute((sq % 7) as u8) }
}


pub fn opp_color(color: Color) -> Color
{
    match color {
        Color::Red => Color::Blue,
        Color::Blue => Color::Red,
        Color::None => Color::None
    }
}

pub fn lsb(x: u64) -> u8 {
    (x & (!x + 1)) as u8
}

pub fn pop_lsb(value: &mut u64) -> u8 {
    assert!(*value > 0);
    let index = value.trailing_zeros() as usize;
    *value &= value.wrapping_sub(1);
    index as u8
}

pub fn char_to_digit(ch: char) -> u8 {
    assert!(ch.is_digit(10));
    ch.to_digit(10).unwrap() as u8
}

pub fn digit_to_char(num: u8) -> char {
    assert!(num <= 9);
    (num + b'0') as char
}

const ADJACENT_OFFSETS: [[i8; 2]; 8] = [
    [0, 1], [0, -1], [1,  0], [-1,  0],
    [1, 1], [1, -1], [-1, 1], [-1, -1]
];

const LEAP_OFFSETS: [[i8; 2]; 16] = [
    [0, 2], [0, -2], [2,  0], [-2,  0],
    [2, 2], [2, -2], [-2, 2], [-2, -2],
    [1, 2], [1, -2], [-1, 2], [-1, -2],
    [2, 1], [2, -1], [-2, 1], [-2, -1],
];

pub fn get_attacks() -> ([u64; 49], [u64; 49])
{
    let mut adjacent_squares_table: [u64; 49] = [0; 49];
    let mut leap_squares_table: [u64; 49] = [0; 49];

    for sq in 0..49
    {
        adjacent_squares_table[sq] = 0;
        let rank: i16 = square_rank(sq as u8) as i16;
        let file: i16 = square_file(sq as u8) as i16;
        // Init adjacent squares for this sq
        for i in 0..8
        {
            let rank2 = rank + ADJACENT_OFFSETS[i][0] as i16;
            let file2 = file + ADJACENT_OFFSETS[i][1] as i16;
            if rank2 >= 0 && rank2 <= 6 && file2 >= 0 && file2 <= 6
            {
                let adjacent_sq: u8 = (rank2 * 7 + file2) as u8;
                adjacent_squares_table[sq] |= 1u64 << adjacent_sq;
            }
        }
        // Init leap squares for this sq
        for i in 0..16
        {
            let rank2 = rank + LEAP_OFFSETS[i][0] as i16;
            let file2 = file + LEAP_OFFSETS[i][1] as i16;
            if rank2 >= 0 && rank2 <= 6 && file2 >= 0 && file2 <= 6
            {
                let leap_sq: u8 = (rank2 * 7 + file2) as u8;
                leap_squares_table[sq] |= 1u64 << leap_sq;
            }
        }
    }

    return (adjacent_squares_table, leap_squares_table)
}

pub fn print_bitboard(bb: u64) {
    // Format as a binary string with the lowest 49 bits
    let bitset = format!("{:049b}", bb);

    for chunk in bitset.chars().collect::<Vec<_>>().chunks(7) {
        let chunk_str: String = chunk.into_iter().rev().collect();
        println!("{}", chunk_str);
    }
}

pub fn str_to_square(sq: &str) -> Square {
    let file = sq.chars().next().unwrap() as usize - 'a' as usize;
    let rank = sq.chars().nth(1).unwrap().to_digit(10).unwrap() as usize - 1;
    (rank * 7 + file) as Square
}

pub fn move_to_str(mov: Move) -> String
{
    assert!(mov != MOVE_NONE);

    if mov == MOVE_PASS {
        return String::from("0000");
    }
    if mov[FROM] == mov[TO] { 
        SQUARE_TO_STR[mov[TO] as usize].to_string() 
    }
    else { 
        SQUARE_TO_STR[mov[FROM] as usize].to_string() + SQUARE_TO_STR[mov[TO] as usize] 
    }
}

pub fn str_to_move(mov: &str) -> Move {
    if mov.len() == 2 {
        let sq: Square = str_to_square(mov);
        return [sq, sq];
    }

    let str_from = &mov[0..2];
    let str_to = &mov[mov.len() - 2..];

    let from: Square = str_to_square(str_from);
    let to: Square = str_to_square(str_to);
    [from, to]
}

pub fn milliseconds_elapsed(start_time: Instant) -> u32 {
    let now = Instant::now();
    now.duration_since(start_time).as_millis() as u32
}

pub fn incremental_sort(moves: &mut MovesArray, num_moves: u8, moves_scores: &mut [u8; 256], i: usize) -> (Move, u8)
{
    for j in ((i+1) as usize)..(num_moves as usize) {
        if moves_scores[j] > moves_scores[i] {
            (moves[i], moves[j]) = (moves[j], moves[i]);
            (moves_scores[i], moves_scores[j]) = (moves_scores[j], moves_scores[i]);
        }
    }

    (moves[i], moves_scores[i])
}

pub fn clamp<T: Ord>(value: T, min: T, max: T) -> T {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}
