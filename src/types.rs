#![allow(dead_code)]
#![allow(unused_variables)]

pub type Square = u8;

pub const SQUARE_NONE: Square = 255;

#[repr(u8)]
#[derive(Clone, Copy)]
#[derive(PartialEq)]
pub enum Color {
    Red  = 0,
    Blue = 1,
    None = 2
}

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum Rank {
    Rank1 = 0,
    Rank2 = 1,
    Rank3 = 2,
    Rank4 = 3,
    Rank5 = 4,
    Rank6 = 5,
    Rank7 = 6
}

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum File {
    A = 0,
    B = 1,
    C = 2,
    D = 3,
    E = 4,
    F = 5,
    G = 6
}

#[repr(u8)]
#[derive(Clone, Copy)]
#[derive(PartialEq)]
pub enum GameResult {
    None = 0,
    WinRed = 1,
    WinBlue = 2,
    Draw = 3
}

impl ToString for GameResult {
    fn to_string(&self) -> String {
        match self {
            GameResult::None => String::from("No result"),
            GameResult::WinRed => String::from("Red wins"),
            GameResult::WinBlue => String::from("Blue wins"),
            GameResult::Draw => String::from("Draw"),
        }
    }
}

pub const START_FEN: &str = "r5b/7/7/7/7/7/b5r r 0 1";
pub const START_FEN2: &str = "r5b/7/2-1-2/7/2-1-2/7/b5r r 0 1";

pub type Move = [Square; 2];
pub const FROM: usize = 0;
pub const TO: usize = 1;
pub const MOVE_NONE: Move = [255, 255];
pub const MOVE_PASS: Move = [254, 254];
pub type MovesArray = [Move; 256];
pub const EMPTY_MOVES_ARRAY: MovesArray = [MOVE_NONE; 256];

pub const INFINITY: i16 = 32000;
pub const MIN_WIN_SCORE: i16 = 31000;

/*
42 43 44 45 46 47 48
35 36 37 38 39 40 41
28 29 30 31 32 33 34
21 22 23 24 25 26 27
14 15 16 17 18 19 20
 7  8  9 10 11 12 13
 0  1  2  3  4  5  6
*/

pub const A1: Square = 0;
pub const B1: Square = 1;
pub const C1: Square = 2;
pub const D1: Square = 3;
pub const E1: Square = 4;
pub const F1: Square = 5;
pub const G1: Square = 6;
pub const A2: Square = 7;
pub const B2: Square = 8;
pub const C2: Square = 9;
pub const D2: Square = 10;
pub const E2: Square = 11;
pub const F2: Square = 12;
pub const G2: Square = 13;
pub const A3: Square = 14;
pub const B3: Square = 15;
pub const C3: Square = 16;
pub const D3: Square = 17;
pub const E3: Square = 18;
pub const F3: Square = 19;
pub const G3: Square = 20;
pub const A4: Square = 21;
pub const B4: Square = 22;
pub const C4: Square = 23;
pub const D4: Square = 24;
pub const E4: Square = 25;
pub const F4: Square = 26;
pub const G4: Square = 27;
pub const A5: Square = 28;
pub const B5: Square = 29;
pub const C5: Square = 30;
pub const D5: Square = 31;
pub const E5: Square = 32;
pub const F5: Square = 33;
pub const G5: Square = 34;
pub const A6: Square = 35;
pub const B6: Square = 36;
pub const C6: Square = 37;
pub const D6: Square = 38;
pub const E6: Square = 39;
pub const F6: Square = 40;
pub const G6: Square = 41;
pub const A7: Square = 42;
pub const B7: Square = 43;
pub const C7: Square = 44;
pub const D7: Square = 45;
pub const E7: Square = 46;
pub const F7: Square = 47;
pub const G7: Square = 48;


