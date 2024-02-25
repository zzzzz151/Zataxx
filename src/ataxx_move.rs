use crate::types::*;
use crate::utils::*;
use std::ops::Index;
use std::fmt;

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct AtaxxMove {
    pub from: Square,
    pub to: Square
}

pub const MOVE_NONE: AtaxxMove = AtaxxMove { from: 50, to: 50 };
pub const MOVE_PASS: AtaxxMove = AtaxxMove { from: 51, to: 51 };

impl AtaxxMove
{
    pub fn double(from: Square, to: Square) -> Self {
        Self { 
            from: from,
            to: to
        }
    }

    pub const fn single(square: Square) -> Self {
        Self {
            from: square,
            to: square
        }
    }

    pub fn from_u16(encoded: u16) -> Self {
        Self {
            from: (encoded & 0b0000_0000_0011_1111) as Square,
            to: (encoded >> 6) as Square
        }
    }

    pub fn from_uai(uai_move: &str) -> AtaxxMove {
        if uai_move == "0000" {
            return MOVE_PASS;
        }

        if uai_move.len() == 2 {
            let sq: Square = str_to_square(uai_move);
            return Self { from: sq, to: sq };
        }

        let str_from = &uai_move[0..2];
        let str_to = &uai_move[uai_move.len() - 2..];
        Self {
            from: str_to_square(str_from),
            to: str_to_square(str_to)
        }
    }

    pub fn to_u16(&self) -> u16 {
        self.from as u16 | ((self.to as u16) << 6)
    }

    pub fn is_single(&self) -> bool {
        self.from == self.to
    }

    pub fn is_double(&self) -> bool {
        self.from != self.to
    }
}

impl fmt::Display for AtaxxMove 
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result 
    {
        assert!(*self != MOVE_NONE);
        if *self == MOVE_PASS {
            write!(f, "0000")
        }
        else if self.is_single() {
            write!(f, "{}", SQUARE_TO_STR[self.to as usize].to_string())
        }
        else {
            write!(f, "{}",
                SQUARE_TO_STR[self.from as usize].to_string() + SQUARE_TO_STR[self.to as usize])
        }
    }
}

pub struct MovesList {
    pub moves: [AtaxxMove; 256],
    num_moves: u8
}

impl MovesList {
    pub fn default() -> Self {
        Self {
            moves: [MOVE_NONE; 256],
            num_moves: 0
        }
    }

    pub fn size(&self) -> u8 {
        self.num_moves
    }

    pub fn clear(&mut self) {
        self.num_moves = 0;
    }

    pub fn add(&mut self, mov: AtaxxMove) {
        assert!(self.num_moves < 255);
        self.moves[self.num_moves as usize] = mov;
        self.num_moves += 1;
    }

    pub fn swap(&mut self, i: usize, j: usize) {
        assert!(i < self.num_moves.into() && j < self.num_moves.into());
        (self.moves[i], self.moves[j]) = (self.moves[j], self.moves[i]);
    }

}

// This allows MovesList[index]
impl Index<usize> for MovesList {
    type Output = AtaxxMove;

    fn index(&self, index: usize) -> &Self::Output {
        assert!(index < self.num_moves.into());
        &self.moves[index]
    }
}
