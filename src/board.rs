use crate::types::*;
use crate::utils::*;
use crate::ataxx_move::*;
use crate::tables::*;
use crate::nnue::*;

#[derive(Copy, Clone)]
pub struct BoardState
{
    pub color: Color,
    pub bitboards: [u64; 2], // [color]
    pub blocked: u64,
    pub plies_since_single: u16,
    pub current_move: u16,
    pub mov: AtaxxMove,
    pub zobrist_hash: u64,
    pub accumulator: Accumulator
}

impl BoardState 
{
    pub fn default() -> Self {
        Self {
            color: Color::None,
            bitboards: [0, 0],
            blocked: 0,
            plies_since_single: 0,
            current_move: 1,
            mov: MOVE_NONE,
            zobrist_hash: 0,
            accumulator: Accumulator::default()
        }
    }
}

#[derive(Clone)]
pub struct Board 
{
    pub state: BoardState,
    pub states: Vec<BoardState>,
    pub nnue: bool
}

impl Board
{
    pub fn new(fen: &str) -> Self
    {
        // Fen: pieces stm halfmove fullmove 
        // r5b/7/7/7/7/7/b5r r 0 1
        // r5b/7/2-1-2/7/2-1-2/7/b5r r 0 1

        let mut board: Board = Self {
            state: BoardState::default(),
            states: Vec::with_capacity(256),
            nnue: true
        };

        let fen = fen.trim().to_string();
        let fen_split: Vec<&str> = fen.split(' ').map(str::trim).collect();
        let fen_rows: Vec<&str> = fen_split[0].split('/').map(str::trim).collect();

        board.state.color = if fen_split[1] == "r" || fen_split[1] == "x" {Color::Red} else {Color::Blue};
        board.state.zobrist_hash ^= ZOBRIST_COLOR[board.state.color as usize];

        board.state.plies_since_single = fen_split[2].parse().unwrap();
        board.state.current_move = fen_split[3].parse().unwrap();

        // Parse fen rows/pieces
        let mut rank: i16 = 6;
        let mut file: i16 = 0;
        for fen_row in &fen_rows {
            for my_char in fen_row.chars() {
                let sq = rank * 7 + file;
                if my_char == 'r' || my_char == 'x' {
                    board.place_piece(Color::Red, sq as Square);
                }
                else if my_char == 'b' || my_char == 'o' {
                    board.place_piece(Color::Blue, sq as Square);
                }
                else if my_char == '-' {
                    board.state.blocked |= 1u64 << sq;
                    board.state.accumulator.activate_blocker(sq as Square);
                }
                else
                {
                    let empty_squares = char_to_digit(my_char);
                    file += (empty_squares - 1) as i16;
                }
                file += 1;
            }
            rank -= 1;
            file = 0;
        }

        board
    }

    pub fn fen(&self) -> String {
        let mut my_fen = String::new();

        for rank in (0..=6).rev() {
            let mut empty_so_far: u8 = 0;
            for file in 0..=6
            {   
                let square = rank * 7 + file;
                let piece = self.at(square);
                if piece != '.' {
                    if empty_so_far > 0 {
                        my_fen.push(digit_to_char(empty_so_far));
                        empty_so_far = 0;
                    }
                    my_fen.push(piece);
                }
                else if file == 6 {
                    my_fen.push(digit_to_char(empty_so_far + 1));
                }
                else {
                    empty_so_far += 1;
                }
            }
            my_fen.push('/');
        }
        my_fen.pop(); // remove last '/'

        my_fen.push(' ');
        my_fen.push(if self.state.color == Color::Red {'x'} else {'o'});

        my_fen.push(' ');
        my_fen += &self.state.plies_since_single.to_string();

        my_fen.push(' ');
        my_fen += &self.state.current_move.to_string();

        my_fen
    }

    pub fn place_piece(&mut self, color: Color, sq: Square)
    {
        self.state.bitboards[color as usize] |= 1u64 << (sq as u8);
        self.state.zobrist_hash ^= ZOBRIST_TABLE[color as usize][sq as usize];
        if self.nnue {
            self.state.accumulator.activate(color, sq);
        }
    }

    pub fn remove_piece(&mut self, color: Color, sq: Square)
    {
        self.state.bitboards[color as usize] ^= 1u64 << (sq as u8);
        self.state.zobrist_hash ^= ZOBRIST_TABLE[color as usize][sq as usize];
        if self.nnue {
            self.state.accumulator.deactivate(color, sq);
        }
    }

    pub fn print(&self) {
        let mut result = String::new();

        for rank in (0..=6).rev() {
            result.push(digit_to_char(rank+1));
            result.push(' ');
            for file in 0..=6 {
                let square = rank * 7 + file;
                result.push(self.at(square as u8));
                result.push(' ');
            }
            result.push('\n');
        }
        result.pop(); // remove last new line

        println!("{}", result);
        println!("  A B C D E F G");
        println!("{}", self.fen());
        println!("Zobrist hash: {}", self.state.zobrist_hash);
    }

    pub fn at(&self, sq: Square) -> char
    {
        let sq_bb: u64 = 1u64 << sq as u8;
        if self.is_square_blocked(sq) {
            '-'
        } else if (self.state.bitboards[Color::Red as usize] & sq_bb) > 0 {
            'x'
        } else if (self.state.bitboards[Color::Blue as usize] & sq_bb) > 0 {
            'o'
        } else {
            '.'
        }
    }

    pub fn is_square_blocked(&self, sq: Square) -> bool {
        (self.state.blocked & (1u64 << sq)) > 0
    }

    pub fn us(&self) -> u64 {
        self.state.bitboards[self.state.color as usize]
    }

    pub fn them(&self) -> u64 {
        self.state.bitboards[opp_color(self.state.color) as usize]
    }

    pub fn occupancy(&self) -> u64 {
        self.state.bitboards[Color::Red as usize] | self.state.bitboards[Color::Blue as usize]
    }

    pub fn swap_stm(&mut self)
    { 
        self.state.zobrist_hash ^= ZOBRIST_COLOR[self.state.color as usize];
        self.state.color = opp_color(self.state.color);
        self.state.zobrist_hash ^= ZOBRIST_COLOR[self.state.color as usize];
    }

    pub fn make_move(&mut self, mov: AtaxxMove)
    {
        assert!(mov != MOVE_NONE);

        self.states.push(self.state);
        self.state.mov = mov;

        if self.state.color == Color::Blue {
            self.state.current_move += 1;
        }

        if mov == MOVE_PASS {
            if self.states.len() > 0 {
                assert!(self.states.last().unwrap().mov != MOVE_PASS);  
            }
            self.state.plies_since_single += 1;
            self.swap_stm();
            return;
        }

        self.place_piece(self.state.color, mov.to);
        if mov.is_double() {
            self.remove_piece(self.state.color, mov.from);
        }

        // Capture enemy pieces adjacent to destination
        let adjacent_squares: u64 = ADJACENT_SQUARES_TABLE[mov.to as usize];
        let opp_stm = opp_color(self.state.color);
        let mut enemies_captured = adjacent_squares & self.state.bitboards[opp_stm as usize];
        while enemies_captured > 0
        {
            let sq_captured: Square = pop_lsb(&mut enemies_captured) as Square;
            self.remove_piece(opp_stm, sq_captured);
            self.place_piece(self.state.color, sq_captured);
        }

        if mov.is_single() {
            self.state.plies_since_single = 0;
        }
        else {
            self.state.plies_since_single += 1;
        }

        self.swap_stm();
    }

    pub fn undo_move(&mut self)
    {
        assert!(self.state.mov != MOVE_NONE);
        assert!(self.states.len() > 0);
        self.state = *self.states.last().unwrap();
        self.states.pop();
    }

    pub fn moves(&mut self, moves_list: &mut MovesList)
    {
        moves_list.clear();
        let mut us = self.us();
        let mut adjacent_target_squares: u64 = 0;

        while us > 0 {
            let from = pop_lsb(&mut us) as Square;
            adjacent_target_squares |= ADJACENT_SQUARES_TABLE[from as usize];
            let mut leap_squares: u64 = LEAP_SQUARES_TABLE[from as usize]
                                        & !self.occupancy()
                                        & !self.state.blocked;

            while leap_squares > 0 {
                let to = pop_lsb(&mut leap_squares) as Square;
                moves_list.add(AtaxxMove::double(from, to));
            }
        }

        adjacent_target_squares &= !self.occupancy() & !self.state.blocked;
        while adjacent_target_squares > 0 {
            let sq = pop_lsb(&mut adjacent_target_squares) as Square;
            moves_list.add(AtaxxMove::single(sq));
        }

        if moves_list.size() == 0 {
            assert!(self.state.mov != MOVE_PASS);
            moves_list.add(MOVE_PASS);
        }
    }
    
    pub fn game_state(&mut self) -> (GameState, Color)
    {
        if self.state.bitboards[Color::Red as usize] == 0 {
            return (GameState::Won, Color::Blue);
        }

        if self.state.bitboards[Color::Blue as usize] == 0 {
            return (GameState::Won, Color::Red);
        }
        
        if self.occupancy().count_ones() == 49 - self.state.blocked.count_ones()
        {
            let num_red_pieces: u8 = self.state.bitboards[Color::Red as usize].count_ones() as u8;
            let num_blue_pieces: u8 = self.state.bitboards[Color::Blue as usize].count_ones() as u8;

            return if num_red_pieces > num_blue_pieces { 
                (GameState::Won, Color::Red)
            } else if num_blue_pieces > num_red_pieces { 
                (GameState::Won, Color::Blue)
            } else { 
                (GameState::Draw, Color::None)
            }
        }

        if !self.must_pass() {
            return if self.state.plies_since_single >= 100 { 
                (GameState::Draw, Color::None)
            } else {
                (GameState::Ongoing, Color::None)
            }
        }

        self.state.color = opp_color(self.state.color);
        let opponent_must_pass: bool = self.must_pass();
        self.state.color = opp_color(self.state.color);

        if !opponent_must_pass {
            return if self.state.plies_since_single >= 100 { 
                (GameState::Draw, Color::None)
            } else {
                (GameState::Ongoing, Color::None)
            }
        }

        let num_red_pieces: u8 = self.state.bitboards[Color::Red as usize].count_ones() as u8;
        let num_blue_pieces: u8 = self.state.bitboards[Color::Blue as usize].count_ones() as u8;

        return if num_red_pieces > num_blue_pieces { 
            (GameState::Won, Color::Red)
        } else if num_blue_pieces > num_red_pieces { 
            (GameState::Won, Color::Blue)
        } else { 
            (GameState::Draw, Color::None)
        }
    }

    pub fn must_pass(&self) -> bool
    {
        let mut us = self.us();
        let mut adjacent_target_squares: u64 = 0;

        while us > 0 {
            let from: Square = pop_lsb(&mut us) as Square;
            adjacent_target_squares |= ADJACENT_SQUARES_TABLE[from as usize];
            let leap_squares: u64 = LEAP_SQUARES_TABLE[from as usize]
                                    & !self.occupancy()
                                    & !self.state.blocked;
            if leap_squares > 0 {
                return false;
            }
        }

        adjacent_target_squares &= !self.occupancy() & !self.state.blocked;
        if adjacent_target_squares > 0 {
            return false;
        }

        true
    }

    pub fn num_adjacent_enemies(&self, sq: Square) -> u8 {
        (ADJACENT_SQUARES_TABLE[sq as usize] & self.them()).count_ones() as u8
    }

    pub fn evaluate(&self) -> i32 {
        evaluate(self.state.color, &self.state.accumulator)
    }

}
