pub const I32_MAX: i32 = 2147483647;
pub const U64_MAX: u64 = 18446744073709551615;
pub const I64_MAX: i64 = 9223372036854775807;

pub const START_FEN: &str = "x5o/7/7/7/7/7/o5x x 0 1";
pub const INFINITY: i32 = 32000;
pub const MIN_WIN_SCORE: i32 = 31000;
pub const EVAL_NONE: i32 = INFINITY;

/*
42 43 44 45 46 47 48
35 36 37 38 39 40 41
28 29 30 31 32 33 34
21 22 23 24 25 26 27
14 15 16 17 18 19 20
 7  8  9 10 11 12 13
 0  1  2  3  4  5  6
*/

pub type Square = u8;
#[allow(dead_code)]
pub const SQUARE_NONE: Square = 255;

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Color {
    Red  = 0,
    Blue = 1,
    None = 2
}

#[repr(u8)]
#[derive(Clone, Copy)]
#[allow(dead_code)]
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
#[allow(dead_code)]
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
#[derive(Clone, Copy, PartialEq, Debug)]
#[allow(dead_code)]
pub enum GameState {
    Ongoing = 0,
    Draw = 1,
    Won = 2
}

/*
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
*/

