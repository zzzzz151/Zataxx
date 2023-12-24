/*
use crate::types::*;
//use crate::utils::*;
use crate::board::*;

#[derive(Clone, Copy)]
#[derive(PartialEq)]
pub enum MovePickerStage {
    TTMove,
    GenerateMoves,
    YieldingMoves
}

pub struct MovePicker {
    pub tt_move: Move,
    pub moves: MovesArray,
    pub num_moves: u8,
    pub scores: [i8; 256],
    pub generated: usize,
    pub stage: MovePickerStage
}

impl MovePicker
{
    pub fn new(tt_move: Move) -> Self 
    {
        Self {
            tt_move: tt_move,
            moves: EMPTY_MOVES_ARRAY,
            num_moves: 1,
            scores: [0; 256],
            generated: 0,
            stage: MovePickerStage::TTMove
        }
    }

    pub fn next(&mut self, board: &mut Board) -> Option<(Move, i8)> // (move, move_score)
    {
        match self.stage 
        {
        MovePickerStage::TTMove => 
        {
            self.stage = MovePickerStage::GenerateMoves;

            if board.is_legal(self.tt_move) {
                self.generated += 1;
                return Some((self.tt_move, 127));
            }
            else {
                self.tt_move = MOVE_NONE;
                return self.next(board);
            }
        }
        MovePickerStage::GenerateMoves => 
        {
            self.stage = MovePickerStage::YieldingMoves;

            if self.tt_move == MOVE_PASS { 
                return None;
            }

            self.num_moves = board.moves(&mut self.moves);
            assert!(self.num_moves > 0);

            if self.num_moves == 1 {
                if self.generated == 1 {
                    return None;
                }
                else {
                    self.generated += 1;
                    return Some((self.moves[0], 0));
                }
            }

            for i in 0..(self.num_moves as usize)
            { 
                let mov: Move = self.moves[i];
                if mov == self.tt_move 
                { 
                    // TT move is first element of self.moves and self.scores
                    (self.moves[0], self.moves[i]) = (self.moves[i], self.moves[0]); 
                    (self.scores[0], self.scores[i]) = (self.scores[i], self.scores[0]); 
                }
                else 
                {
                    self.scores[i] = (mov[TO] == mov[FROM]) as i8; // bonus for singles
                    self.scores[i] += board.num_adjacent_enemies(mov[TO]) as i8; // bonus for pieces captured
                }
            }

            return self.next(board);
        }
        MovePickerStage::YieldingMoves => 
        {
            if self.generated >= self.num_moves as usize {
                return None;
            }
            
            // Incremental sort
            for j in (self.generated + 1)..(self.num_moves as usize) 
            {
                if self.scores[j] > self.scores[self.generated] 
                {
                    // Swap moves
                    (self.moves[self.generated], self.moves[j]) 
                        = (self.moves[j], self.moves[self.generated]);

                    // Swap moves scores
                    (self.scores[self.generated], self.scores[j]) 
                        = (self.scores[j], self.scores[self.generated]);
                }
            }

            self.generated += 1;
            Some((self.moves[self.generated - 1], self.scores[self.generated - 1]))
        }
        }
    }

}
*/