use crate::types::*;
use crate::utils::*;
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
    pub mov: Move,
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

pub struct Board 
{
    pub state: BoardState,
    pub states: Vec<BoardState>
}

impl Board
{
    pub fn default() -> Self {
        Self {
            state: BoardState::default(),
            states: Vec::with_capacity(256)
        }
    }

    pub fn new(fen: &str) -> Self
    {
        // Fen: pieces stm halfmove fullmove 
        // r5b/7/7/7/7/7/b5r r 0 1
        // r5b/7/2-1-2/7/2-1-2/7/b5r r 0 1

        let mut board: Board = Board::default();
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
        my_fen.push(if self.state.color == Color::Red {'r'} else {'b'});

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
        self.state.accumulator.update(color, sq, true);
    }

    pub fn remove_piece(&mut self, color: Color, sq: Square)
    {
        self.state.bitboards[color as usize] ^= 1u64 << (sq as u8);
        self.state.zobrist_hash ^= ZOBRIST_TABLE[color as usize][sq as usize];
        self.state.accumulator.update(color, sq, false);
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
        }
        else if (self.state.bitboards[Color::Red as usize] & sq_bb) > 0 {
            'r'
        }
        else if (self.state.bitboards[Color::Blue as usize] & sq_bb) > 0 {
            'b'
        }
        else {
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

    pub fn make_move(&mut self, mov: Move)
    {
        assert!(mov != MOVE_NONE);

        self.states.push(self.state);

        if self.state.color == Color::Blue {
            self.state.current_move += 1;
        }

        if mov == MOVE_PASS {
            self.state.plies_since_single += 1;
            self.swap_stm();
            return
        }

        // create piece on destination
        self.place_piece(self.state.color, mov[TO]);

        // if destination is not adjacent to source, remove piece at source
        if mov[FROM] != mov[TO] {
            self.remove_piece(self.state.color, mov[FROM]);
        }

        // Capture enemy pieces adjacent to destination
        let adjacent_squares: u64 = ADJACENT_SQUARES_TABLE[mov[TO] as usize];
        let opp_stm = opp_color(self.state.color);
        let mut enemies_captured = adjacent_squares & self.state.bitboards[opp_stm as usize];
        while enemies_captured > 0
        {
            let sq_captured: Square = pop_lsb(&mut enemies_captured) as Square;
            self.remove_piece(opp_stm, sq_captured);
            self.place_piece(self.state.color, sq_captured);
        }

        if mov[FROM] == mov[TO] {
            self.state.plies_since_single = 0;
        }
        else {
            self.state.plies_since_single += 1;
        }

        self.swap_stm();
    }

    pub fn undo_move(&mut self)
    {
        assert!(self.states.len() > 0);
        self.state = *self.states.last().unwrap();
        self.states.pop();
    }

    pub fn moves(&mut self, moves: &mut MovesArray) -> u8
    {
        let mut num_moves: u8 = 0;
        let mut us = self.us();
        let mut adjacent_target_squares: u64 = 0;

        while us > 0
        {
            let from: Square = pop_lsb(&mut us) as Square;
            adjacent_target_squares |= ADJACENT_SQUARES_TABLE[from as usize];
            let mut leap_squares: u64 = LEAP_SQUARES_TABLE[from as usize]
                                        & !self.occupancy()
                                        & !self.state.blocked;
            while leap_squares > 0 {
                let to: Square = pop_lsb(&mut leap_squares) as Square;
                moves[num_moves as usize] = [from, to];
                num_moves += 1;
            }
        }

        adjacent_target_squares &= !self.occupancy() & !self.state.blocked;
        while adjacent_target_squares > 0
        {
            let to: Square = pop_lsb(&mut adjacent_target_squares);
            moves[num_moves as usize] = [to, to];
            num_moves += 1;
        }

        if num_moves == 0
        {
            if self.states.len() > 0 && self.states.last().unwrap().mov == MOVE_PASS {
                return 0;
            }

            moves[0] = MOVE_PASS;
            return 1;
        }

        return num_moves;
    }
    
    pub fn get_game_result(&mut self) -> GameResult
    {
        if self.state.bitboards[Color::Red as usize] == 0 {
            return GameResult::WinBlue;
        }

        if self.state.bitboards[Color::Blue as usize] == 0 {
            return GameResult::WinRed;
        }
        
        if self.occupancy().count_ones() == 49 - self.state.blocked.count_ones()
        {
            let num_red_pieces: u8 = self.state.bitboards[Color::Red as usize].count_ones() as u8;
            let num_blue_pieces: u8 = self.state.bitboards[Color::Blue as usize].count_ones() as u8;

            return if num_red_pieces > num_blue_pieces 
                       { GameResult::WinRed }
                   else if num_blue_pieces > num_red_pieces 
                       { GameResult::WinBlue }
                   else 
                       { GameResult::Draw }
        }

        if !self.must_pass() {
            return if self.state.plies_since_single >= 100 { GameResult::Draw } else { GameResult::None };
        }

        self.state.color = opp_color(self.state.color);
        let opponent_must_pass: bool = self.must_pass();
        self.state.color = opp_color(self.state.color);

        if !opponent_must_pass {
            return if self.state.plies_since_single >= 100 { GameResult::Draw } else { GameResult::None };
        }

        let num_red_pieces: u8 = self.state.bitboards[Color::Red as usize].count_ones() as u8;
        let num_blue_pieces: u8 = self.state.bitboards[Color::Blue as usize].count_ones() as u8;

        return if num_red_pieces > num_blue_pieces 
                   { GameResult::WinRed }
               else if num_blue_pieces > num_red_pieces 
                   { GameResult::WinBlue }
               else 
                   { GameResult::Draw }
    }

    pub fn must_pass(&self) -> bool
    {
        let mut us = self.us();
        let mut adjacent_target_squares: u64 = 0;

        while us > 0
        {
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

}
