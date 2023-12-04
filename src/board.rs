#![allow(dead_code)]
#![allow(unused_variables)]

use crate::types::*;
use crate::utils::*;
use crate::search::*;

#[derive(Clone)]
pub struct Board 
{
    pub color: Color,
    pub bitboards: [u64; 2], // [color]
    pub blocked: u64,
    pub plies_since_capture: u16,
    pub current_move: u16,
    pub states: Vec<Board>, 
    pub mov: Move,
}

impl Board
{
    pub fn default() -> Self {
        Self {
            color: Color::Red,
            bitboards: [0, 0],
            blocked: 0,
            plies_since_capture: 0,
            current_move: 1,
            states: Vec::new(),
            mov: MOVE_NONE,
        }
    }

    pub fn new(fen: &str) -> Self
    {
        // Fen: pieces stm halfmove fullmove 
        // r5b/7/7/7/7/7/b5r x 0 1
        // r5b/7/2-1-2/7/2-1-2/7/b5r x 0 1

        let mut board = Board::default();
        let fen = fen.trim().to_string();
        let fen_split: Vec<&str> = fen.split(' ').map(str::trim).collect();
        let fen_rows: Vec<&str> = fen_split[0].split('/').map(str::trim).collect();

        board.color = if fen_split[1] == "r" || fen_split[1] == "x" {Color::Red} else {Color::Blue};
        board.plies_since_capture = fen_split[2].parse().unwrap();
        board.current_move = fen_split[3].parse().unwrap();

        // Parse fen rows/pieces
        let mut rank: i16 = 6;
        let mut file: i16 = 0;
        for fen_row in &fen_rows {
            for my_char in fen_row.chars() {
                let sq = rank * 7 + file;
                if my_char == 'r' || my_char == 'x' {
                    board.bitboards[Color::Red as usize] |= 1u64 << sq;
                }
                else if my_char == 'b' || my_char == 'o' {
                    board.bitboards[Color::Blue as usize] |= 1u64 << sq;
                }
                else if my_char == '-' {
                    board.blocked |= 1u64 << sq;
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
                if piece != '.'
                {
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
        my_fen.push(if self.color == Color::Red {'r'} else {'b'});

        my_fen.push(' ');
        my_fen += &self.plies_since_capture.to_string();

        my_fen.push(' ');
        my_fen += &self.current_move.to_string();

        my_fen
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
    }

    pub fn at(&self, sq: Square) -> char
    {
        let sq_bb: u64 = 1u64 << sq as u8;
        if self.is_square_blocked(sq) {
            '-'
        }
        else if (self.bitboards[Color::Red as usize] & sq_bb) > 0 {
            'r'
        }
        else if (self.bitboards[Color::Blue as usize] & sq_bb) > 0 {
            'b'
        }
        else {
            '.'
        }
    }

    pub fn is_square_blocked(&self, sq: Square) -> bool {
        (self.blocked & (1u64 << sq)) > 0
    }

    pub fn us(&self) -> u64 {
        self.bitboards[self.color as usize]
    }

    pub fn them(&self) -> u64 {
        self.bitboards[opp_color(self.color) as usize]
    }

    pub fn make_move(&mut self, mov: Move)
    {
        assert!(mov != MOVE_NONE);

        self.states.push(self.clone());

        if self.color == Color::Blue {
            self.current_move += 1;
        }

        if mov == MOVE_PASS {
            self.color = opp_color(self.color);
            return
        }

        // create piece on destination
        self.bitboards[self.color as usize] |= 1u64 << mov[TO];

        let adjacent_squares_table = ADJACENT_SQUARES_TABLE.lock().unwrap();
        let adjacent_squares: u64 = adjacent_squares_table[mov[TO] as usize];

        // if destination is not adjacent to source, remove piece at source
        if mov[FROM] != mov[TO] {
            self.bitboards[self.color as usize] ^= 1u64 << mov[FROM];
        }

        // Capture enemy pieces adjacent to destination
        let enemies_captured = adjacent_squares & self.bitboards[opp_color(self.color) as usize];
        self.bitboards[self.color as usize] |= enemies_captured;
        self.bitboards[opp_color(self.color) as usize] ^= enemies_captured;

        if enemies_captured > 0 {
            self.plies_since_capture = 0;
        }
        else {
            self.plies_since_capture += 1;
        }

        self.color = opp_color(self.color);
    }

    pub fn undo_move(&mut self)
    {
        assert!(self.states.len() > 0);
        let last_state: &Board = self.states.last().unwrap();
        self.color = last_state.color;
        self.bitboards = last_state.bitboards;
        self.blocked = last_state.blocked;
        self.plies_since_capture = last_state.plies_since_capture;
        self.current_move = last_state.current_move;
        self.mov = last_state.mov;
        self.states.pop();
    }

    pub fn occupancy(&self) -> u64 {
        self.bitboards[Color::Red as usize] | self.bitboards[Color::Blue as usize]
    }

    pub fn moves(&mut self, moves: &mut [Move; 256]) -> u8
    {
        let mut num_moves: u8 = 0;

        let adjacent_squares_table = ADJACENT_SQUARES_TABLE.lock().unwrap();
        let leap_squares_table = LEAP_SQUARES_TABLE.lock().unwrap();

        let mut us = self.us();
        let mut adjacent_target_squares: u64 = 0;

        while us > 0
        {
            let from: Square = pop_lsb(&mut us) as Square;
            adjacent_target_squares |= adjacent_squares_table[from as usize];
            let mut leap_squares: u64 = leap_squares_table[from as usize]
                                        & !self.occupancy()
                                        & !self.blocked;
            while leap_squares > 0 {
                let to: Square = pop_lsb(&mut leap_squares) as Square;
                moves[num_moves as usize] = [from, to];
                num_moves += 1;
            }
        }

        adjacent_target_squares &= !self.occupancy() & !self.blocked;
        while adjacent_target_squares > 0
        {
            let to: Square = pop_lsb(&mut adjacent_target_squares);
            moves[num_moves as usize] = [to, to];
            num_moves += 1;
        }

        if num_moves == 0
        {
            moves[0] = MOVE_PASS;
            num_moves = 1;
        }

        return num_moves;
    }
 
    pub fn get_game_result(&mut self) -> GameResult
    {
        if self.bitboards[Color::Red as usize] == 0 {
            return GameResult::WinBlue;
        }

        if self.bitboards[Color::Blue as usize] == 0 {
            return GameResult::WinRed;
        }

        let max_squares_occupied: u8 = (49 - self.blocked.count_ones()) as u8;
        let num_red_pieces: u8 = self.bitboards[Color::Red as usize].count_ones() as u8;
        if num_red_pieces == max_squares_occupied {
            return GameResult::WinRed;
        }

        let num_blue_pieces: u8 = self.bitboards[Color::Blue as usize].count_ones() as u8;
        if num_blue_pieces == max_squares_occupied {
            return GameResult::WinBlue;
        }

        if self.occupancy().count_ones() == max_squares_occupied.into() 
        {
            return if num_red_pieces > num_blue_pieces { GameResult::WinRed }
                    else if num_blue_pieces > num_red_pieces { GameResult::WinBlue }
                    else { GameResult::Draw }
        }

        if self.plies_since_capture >= 100 {
            return GameResult::Draw;
        }

        let mut moves: [Move; 256] = [MOVE_NONE; 256];
        self.moves(&mut moves);
        if moves[0] != MOVE_PASS {
            return GameResult::None
        }
        
        self.make_move(MOVE_PASS);
        self.moves(&mut moves);
        self.undo_move();
        if moves[0] != MOVE_PASS {
            return GameResult::None
        }

        return if num_red_pieces > num_blue_pieces { GameResult::WinRed }
               else if num_blue_pieces > num_red_pieces { GameResult::WinBlue }
               else { GameResult::Draw }
    }

    pub fn eval(&mut self) -> i16
    {
        self.us().count_ones() as i16 - self.them().count_ones() as i16
    }
}
