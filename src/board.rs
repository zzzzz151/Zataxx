use crate::types::*;
use crate::utils::*;
use crate::ataxx_move::*;
use arrayvec::ArrayVec;

#[derive(Copy, Clone)]
pub struct BoardState
{
    pub color: Color,
    pub bitboards: [u64; 2], // [color]
    pub gaps: u64,
    pub plies_since_single: u16,
    pub move_counter: u16,
    pub mov: AtaxxMove,
    pub zobrist_hash: u64,
}

impl BoardState 
{
    pub fn default() -> Self {
        Self {
            color: Color::None,
            bitboards: [0, 0],
            gaps: 0,
            plies_since_single: 0,
            move_counter: 1,
            mov: MOVE_NONE,
            zobrist_hash: 0,
        }
    }
    
    pub fn new(fen: &str) -> Self {
        // Fen: pieces stm halfmove fullmove 
        // r5b/7/7/7/7/7/b5r r 0 1
        // r5b/7/2-1-2/7/2-1-2/7/b5r r 0 1

        let mut board_state: BoardState = Self::default();

        let fen = fen.trim().to_string();
        let fen_split: Vec<&str> = fen.split(' ').map(str::trim).collect();
        let fen_rows: Vec<&str> = fen_split[0].split('/').map(str::trim).collect();

        board_state.color = if fen_split[1] == "r" || fen_split[1] == "x" {Color::Red} else {Color::Blue};
        board_state.zobrist_hash ^= ZOBRIST_COLOR[board_state.color as usize];

        board_state.plies_since_single = fen_split[2].parse().unwrap();
        board_state.move_counter = fen_split[3].parse().unwrap();

        // Parse fen rows/pieces
        let mut rank: i16 = 6;
        let mut file: i16 = 0;
        for fen_row in &fen_rows {
            for my_char in fen_row.chars() {
                let sq = rank * 7 + file;
                if my_char == 'r' || my_char == 'x' {
                    board_state.place_piece(Color::Red, sq as Square);
                }
                else if my_char == 'b' || my_char == 'o' {
                    board_state.place_piece(Color::Blue, sq as Square);
                }
                else if my_char == '-' {
                    board_state.gaps |= 1u64 << sq;
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

        board_state
    }

    pub fn place_piece(&mut self, color: Color, sq: Square)
    {
        self.bitboards[color as usize] |= 1u64 << (sq as u8);
        self.zobrist_hash ^= ZOBRIST_TABLE[color as usize][sq as usize];
    }

    pub fn remove_piece(&mut self, color: Color, sq: Square)
    {
        self.bitboards[color as usize] ^= 1u64 << (sq as u8);
        self.zobrist_hash ^= ZOBRIST_TABLE[color as usize][sq as usize];
    }

    pub fn occupancy(&self) -> u64 {
        self.bitboards[0] | self.bitboards[1]
    }

    pub fn switch_stm(&mut self)
    { 
        self.zobrist_hash ^= ZOBRIST_COLOR[self.color as usize];
        self.color = opp_color(self.color);
        self.zobrist_hash ^= ZOBRIST_COLOR[self.color as usize];
    }

    pub fn is_gap(&self, sq: Square) -> bool {
        (self.gaps & (1u64 << sq)) > 0
    }

    pub fn piece_at(&self, sq: Square) -> char
    {
        let sq_bb: u64 = 1u64 << sq as u8;
        if self.is_gap(sq) {
            '-'
        } else if (self.bitboards[Color::Red as usize] & sq_bb) > 0 {
            'x'
        } else if (self.bitboards[Color::Blue as usize] & sq_bb) > 0 {
            'o'
        } else {
            ' '
        }
    }

    pub fn fen(&self) -> String {
        let mut my_fen = String::new();

        for rank in (0..=6).rev() {
            let mut empty_so_far: u8 = 0;
            for file in 0..=6
            {   
                let square = rank * 7 + file;
                let piece: char = self.piece_at(square);
                if piece != ' ' {
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
        my_fen.push(if self.color == Color::Red {'x'} else {'o'});

        my_fen.push(' ');
        my_fen += &self.plies_since_single.to_string();

        my_fen.push(' ');
        my_fen += &self.move_counter.to_string();

        my_fen
    }

    pub fn make_move(&mut self, mov: AtaxxMove)
    {
        assert!(mov != MOVE_NONE);
        
        self.mov = mov;

        if self.color == Color::Blue {
            self.move_counter += 1;
        }

        if mov == MOVE_PASS {
            self.plies_since_single += 1;
            self.switch_stm();
            return;
        }

        self.place_piece(self.color, mov.to);
        if mov.is_double() {
            self.remove_piece(self.color, mov.from);
        }

        // Capture enemy pieces adjacent to destination
        let adjacent_squares: u64 = ADJACENT[mov.to as usize];
        let opp_stm = opp_color(self.color);
        let mut enemies_captured = adjacent_squares & self.bitboards[opp_stm as usize];
        while enemies_captured > 0
        {
            let sq_captured: Square = pop_lsb(&mut enemies_captured) as Square;
            self.remove_piece(opp_stm, sq_captured);
            self.place_piece(self.color, sq_captured);
        }

        if mov.is_single() {
            self.plies_since_single = 0;
        }
        else {
            self.plies_since_single += 1;
        }

        self.switch_stm();
    }

    pub fn moves(&mut self, moves: &mut ArrayVec<AtaxxMove, 256>)
    {
        moves.clear();
        let mut us = self.bitboards[self.color as usize];
        let mut adjacent_target_squares: u64 = 0;

        while us > 0 {
            let from = pop_lsb(&mut us) as Square;
            adjacent_target_squares |= ADJACENT[from as usize];
            let mut doubles: u64 = DOUBLES[from as usize] & !self.occupancy() & !self.gaps;

            while doubles > 0 {
                let to = pop_lsb(&mut doubles) as Square;
                moves.push(AtaxxMove::double(from, to));
            }
        }

        adjacent_target_squares &= !self.occupancy() & !self.gaps;
        while adjacent_target_squares > 0 {
            let sq = pop_lsb(&mut adjacent_target_squares) as Square;
            moves.push(AtaxxMove::single(sq));
        }

        if moves.len() == 0 {
            assert!(self.mov != MOVE_PASS);
            moves.push(MOVE_PASS);
        }
    }

    pub fn game_state(&mut self) -> (GameState, Color)
    {
        if self.bitboards[Color::Red as usize] == 0 {
            return (GameState::Won, Color::Blue);
        }

        if self.bitboards[Color::Blue as usize] == 0 {
            return (GameState::Won, Color::Red);
        }
        
        if self.occupancy().count_ones() == 49 - self.gaps.count_ones()
        {
            let num_red_pieces: u8 = self.bitboards[Color::Red as usize].count_ones() as u8;
            let num_blue_pieces: u8 = self.bitboards[Color::Blue as usize].count_ones() as u8;

            return if num_red_pieces > num_blue_pieces { 
                (GameState::Won, Color::Red)
            } else if num_blue_pieces > num_red_pieces { 
                (GameState::Won, Color::Blue)
            } else { 
                (GameState::Draw, Color::None)
            }
        }

        if !self.must_pass() {
            return if self.plies_since_single >= 100 { 
                (GameState::Draw, Color::None)
            } else {
                (GameState::Ongoing, Color::None)
            }
        }

        self.color = opp_color(self.color);
        let opponent_must_pass: bool = self.must_pass();
        self.color = opp_color(self.color);

        if !opponent_must_pass {
            return if self.plies_since_single >= 100 { 
                (GameState::Draw, Color::None)
            } else {
                (GameState::Ongoing, Color::None)
            }
        }

        let num_red_pieces: u8 = self.bitboards[Color::Red as usize].count_ones() as u8;
        let num_blue_pieces: u8 = self.bitboards[Color::Blue as usize].count_ones() as u8;

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
        let mut us = self.bitboards[self.color as usize];
        let mut adjacent_target_squares: u64 = 0;

        while us > 0 {
            let from: Square = pop_lsb(&mut us) as Square;
            adjacent_target_squares |= ADJACENT[from as usize];
            let doubles: u64 = DOUBLES[from as usize] & !self.occupancy() & !self.gaps;

            if doubles > 0 { return false; }
        }

        adjacent_target_squares &= !self.occupancy() & !self.gaps;
        if adjacent_target_squares > 0 {
            return false;
        }

        true
    }
}

pub struct Board 
{
    state: *mut BoardState,
    states: Vec<BoardState>,
}

impl Board
{
    pub fn new(fen: &str) -> Self
    {
        let mut board: Board = Self {
            state: std::ptr::null_mut(),
            states: Vec::with_capacity(256),
        };

        board.states.push(BoardState::new(fen));
        board.state = board.states.last_mut().unwrap() as *mut _;
        board
    }

    pub fn num_states(&self) -> usize {
        self.states.len()
    }

    pub fn side_to_move(&self) -> Color {
        unsafe { (*self.state).color }
    }

    pub fn bitboards(&self) -> [u64; 2] {
        unsafe { (*self.state).bitboards }
    }
    
    pub fn red(&self) -> u64 {
        unsafe { (*self.state).bitboards[Color::Red as usize] }
    }

    pub fn blue(&self) -> u64 {
        unsafe { (*self.state).bitboards[Color::Blue as usize] }
    }
    
    #[allow(dead_code)]
    pub fn us(&self) -> u64 { 
        unsafe { (*self.state).bitboards[self.side_to_move() as usize] }
    }
    
    pub fn them(&self) -> u64 {
        unsafe { (*self.state).bitboards[opp_color(self.side_to_move()) as usize] }
    }
    
    #[allow(dead_code)]
    pub fn occupancy(&self) -> u64 {
        unsafe { (*self.state).occupancy() }
    }
    
    pub fn last_move(&self) -> AtaxxMove {
        unsafe { (*self.state).mov }
    }

    pub fn zobrist_hash(&self) -> u64 {
        unsafe { (*self.state).zobrist_hash }
    }

    pub fn plies_since_single(&self) -> u16 {
        unsafe { (*self.state).plies_since_single }
    }

    pub fn place_piece(&mut self, color: Color, sq: Square)
    {
        unsafe { (*self.state).place_piece(color, sq); }
    }

    pub fn remove_piece(&mut self, color: Color, sq: Square)
    {
        unsafe { (*self.state).remove_piece(color, sq); }
    }

    #[allow(dead_code)]
    pub fn switch_stm(&mut self) {
        unsafe { (*self.state).switch_stm() }
    }

    pub fn color_at(&self, sq: Square) -> Color {
        let sq_bb: u64 = 1u64 << sq as u8;

        if (self.red() & sq_bb) > 0 {
            Color::Red
        } else if (self.blue() & sq_bb) > 0 {
            Color::Blue
        }
        else {
            Color::None
        }
    }

    pub fn piece_at(&self, sq: Square) -> char {
        unsafe { (*self.state).piece_at(sq) }
    }

    pub fn fen(&self) -> String {
        unsafe { (*self.state).fen() }
    }

    pub fn moves(&mut self, moves: &mut ArrayVec<AtaxxMove, 256>) {
        unsafe { (*self.state).moves(moves); }
    }

    pub fn game_state(&mut self) -> (GameState, Color) {
        unsafe { (*self.state).game_state() }
    }

    pub fn must_pass(&self) -> bool {
        unsafe { (*self.state).must_pass() }
    }

    pub fn num_adjacent_enemies(&self, sq: Square) -> u8 
    {
        (ADJACENT[sq as usize] & self.them()).count_ones() as u8
    }

    pub fn make_move(&mut self, mov: AtaxxMove) {
        unsafe { self.states.push(*self.state); }
        self.state = self.states.last_mut().unwrap() as *mut _;
        unsafe { (*self.state).make_move(mov); }
    }

    pub fn undo_move(&mut self)
    {
        assert!(self.states.len() > 1 && self.last_move() != MOVE_NONE);
        self.state = std::ptr::null_mut();
        self.states.pop();
        self.state = self.states.last_mut().unwrap() as *mut _;
    }
}

impl Clone for Board {
    fn clone(&self) -> Self {
        let mut cloned = Self {
            state: std::ptr::null_mut(),
            states: self.states.clone(),
        };

        assert!(cloned.states.len() >= 1);
        cloned.state = cloned.states.last_mut().unwrap() as *mut _;
        cloned
    }
}