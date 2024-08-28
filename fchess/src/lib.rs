mod chess_data;
use chess_data::generate_data;

mod data;
use data::*;

const EMPTY_STRING: String = String::new();

#[derive(Clone, Copy)]
pub struct Board {
    bitboards: [BitBoard; 12],
    lookup_tables: [[u64; 64]; 10],
    other_tables: RaycastTables, // This should be in some kind of meta object.

    turn: Color,
}

impl Default for Board {
    fn default() -> Self {
        Board {
            bitboards: STARTING_POSITION,

            lookup_tables: generate_data(),
            other_tables: RaycastTables::new(),

            turn: Color::White,
        }
    }
}

impl Board {
    pub fn print_board(&self) {
        let mut text_representation = self.get_text_representation();
        text_representation.reverse();
        for i in 0..64 {
            let char = text_representation[i].clone();
            if i % 8 == 0 {
                println!();
            }
            if char != String::default() {
                print!("{} ", char);
            } else {
                print!("  ");
            }
        }
    }
    pub fn is_in_checkmate(&self) -> bool {
        let mut move_bitmask = 0;
        for possible_move in 0..64 {
            move_bitmask |= self.generate_legal_moves(possible_move).0;
        }

        move_bitmask == 0
    }
    pub fn clear_square(&mut self, position: u8) {
        for bitboard in &mut self.bitboards {
            bitboard.clear_bit(position);
        }
    }

    pub fn find_piece(&self, position: u8) -> Pieces {
        let mut piece_type = 0;
        for index in 0..12 {
            if self.bitboards[index].get_bit(position) {
                piece_type = index as u8;
                break;
            }
            piece_type = 12;
        }

        Pieces::from_u8(piece_type)
    }

    pub fn switch_turn(&self) -> Color {
        match self.turn {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }

    pub fn move_piece(&self, position: u8, destination: u8) -> Board {
        let mut new_board = *self;

        let piece_type = new_board.find_piece(position);
        new_board.clear_square(position);
        new_board.clear_square(destination);
        new_board.bitboards[piece_type as usize].set_bit(destination);
        new_board.turn = self.switch_turn();

        new_board
    }

    fn raycast_calculate(&self, position: u8, occupancy: u64) -> u64 {
        fn msb(x: u64) -> u32 {
            63 - x.leading_zeros()
        }
        fn lsb(x: u64) -> u32 {
            x.trailing_zeros()
        }
        let mut ray_cast_sum = 0;
        for (ray_id, ray_table) in [
            self.other_tables.north_west,
            self.other_tables.north,
            self.other_tables.north_east,
            self.other_tables.west,
            self.other_tables.east,
            self.other_tables.south_west,
            self.other_tables.south,
            self.other_tables.south_east,
        ]
        .iter()
        .enumerate()
        {
            let mut ray = occupancy & ray_table[position as usize];
            if ray != 0 {
                let r_pos = match ray_id {
                    0..4 => lsb(ray),
                    4..8 => msb(ray),
                    _ => panic!(),
                };

                ray = ray_table[r_pos as usize] ^ ray_table[position as usize];
                ray_cast_sum |= ray;
            } else {
                ray_cast_sum |= ray_table[position as usize];
            }
        }

        ray_cast_sum
    }

    pub fn get_pseudolegal_moves(&self, position: u8) -> BitBoard {
        let piece_type = self.find_piece(position);
        let piece_type_id = piece_type as usize;

        if piece_type == Pieces::None {
            // This is an empty square, no movement.
            return BitBoard(0);
        }

        let piece_color = match piece_type_id / 6 {
            0..1 => Color::White,
            1..2 => Color::Black,
            _ => panic!(),
        };
        if piece_color != self.turn {
            return BitBoard(0);
        }

        let lookup = [0, 1, 2, 3, 4, 9, 0, 1, 2, 3, 4, 9];

        let mut movement = BitBoard(self.lookup_tables[lookup[piece_type_id]][position as usize]);

        let mut friendly_occupancy = 0;
        let mut enemy_occupancy = 0;
        for index in 0..6 {
            friendly_occupancy |= self.bitboards[index].0;
        }
        for index in 6..12 {
            enemy_occupancy |= self.bitboards[index].0;
        }

        if self.turn == Color::Black {
            std::mem::swap(&mut friendly_occupancy, &mut enemy_occupancy);
        }

        let occupancy = friendly_occupancy | enemy_occupancy;

        match piece_type {
            Pieces::WhitePawn => {
                movement.0 |= self.lookup_tables[LookupTable::WhitePawnMoves as usize]
                    [position as usize]
                    & (UNIVERSE ^ occupancy);

                movement.0 |= self.lookup_tables[LookupTable::WhitePawnCaptures as usize]
                    [position as usize]
                    & enemy_occupancy;
            }
            Pieces::BlackPawn => {
                movement.0 |= self.lookup_tables[LookupTable::BlackPawnMoves as usize]
                    [position as usize]
                    & (UNIVERSE ^ occupancy);

                movement.0 |= self.lookup_tables[LookupTable::BlackPawnCaptures as usize]
                    [position as usize]
                    & enemy_occupancy;
            }
            _ => {}
        }

        let is_sliding_piece: bool = match Pieces::from_u8(piece_type as u8) {
            Pieces::WhiteQueen
            | Pieces::WhiteRook
            | Pieces::WhiteBishop
            | Pieces::WhitePawn
            | Pieces::BlackQueen
            | Pieces::BlackRook
            | Pieces::BlackBishop
            | Pieces::BlackPawn => true,

            Pieces::WhiteKnight | Pieces::BlackKnight | Pieces::WhiteKing | Pieces::BlackKing => {
                false
            }

            Pieces::None => panic!(),
        };

        if is_sliding_piece {
            movement.0 &= self.raycast_calculate(position, occupancy);
        }

        movement.0 &= UNIVERSE ^ friendly_occupancy;

        movement
    }

    pub fn try_make_move(&mut self, position: u8, destination: u8) {
        let legal_moves = self.generate_legal_moves(position);

        if legal_moves.0 & (1 << destination) != 0 {
            *self = self.move_piece(position, destination);
        }
    }
    pub fn generate_legal_moves(&self, position: u8) -> BitBoard {
        let psuedo_legal_moves = self.get_pseudolegal_moves(position);

        let mut legal_move_bitmask = 0;

        for bit in 0..64 {
            let bit_bitmask = 1 << bit;

            if psuedo_legal_moves.0 & bit_bitmask != 0 {
                let chess_move = self.move_piece(position, bit);

                let king_bitmask = match chess_move.switch_turn() {
                    Color::White => chess_move.bitboards[0],
                    Color::Black => chess_move.bitboards[6],
                };

                let mut enemy_bitmask = 0;
                for enemy_bit in 0..64 {
                    enemy_bitmask |= chess_move.get_pseudolegal_moves(enemy_bit).0;
                }

                if (enemy_bitmask & king_bitmask.0) == 0 {
                    legal_move_bitmask |= 1 << bit;
                }
            }
        }

        BitBoard(legal_move_bitmask)
    }

    pub fn get_text_representation(&self) -> [String; 64] {
        fn insert_chess_pieces(
            bitboard: BitBoard,
            character_represetation: &str,
            text_representation: &mut [String; 64],
        ) {
            for i in 0..64 {
                if bitboard.get_bit(i) {
                    text_representation[i as usize] = character_represetation.to_string();
                }
            }
        }

        let mut text_representation: [String; 64] = [EMPTY_STRING; 64];

        let characters = ["♚", "♛", "♜", "♝", "♞", "♟", "♔", "♕", "♖", "♗", "♘", "♙"]; // Sorted by material value
        for (index, character) in characters.iter().enumerate() {
            insert_chess_pieces(self.bitboards[index], character, &mut text_representation);
        }

        text_representation
    }
}
