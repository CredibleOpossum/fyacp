mod chess_data;
use std::path::absolute;

use chess_data::generate_data;

mod data;
use data::*;

const EMPTY_STRING: String = String::new();
#[derive(Clone, Copy)]
pub struct CastlingRights {
    white_queenside: bool,
    white_kingside: bool,
    black_queenside: bool,
    black_kingside: bool,
}
impl Default for CastlingRights {
    fn default() -> Self {
        CastlingRights {
            white_queenside: true,
            white_kingside: true,
            black_queenside: true,
            black_kingside: true,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Board {
    bitboards: [BitBoard; 12],
    castling_rights: CastlingRights,
    en_passant: Option<u8>, // Denotes the position of where the en passant square can be captured
    turn: Color,

    lookup_tables: [[u64; 64]; 10], // This should be in some kind of meta object, not related directly to the rules/behavior of chess.
    other_tables: RaycastTables,
}

impl Default for Board {
    fn default() -> Self {
        Board {
            bitboards: STARTING_POSITION,
            castling_rights: CastlingRights::default(),
            en_passant: None,
            turn: Color::White,

            lookup_tables: generate_data(),
            other_tables: RaycastTables::new(),
        }
    }
}

impl Board {
    pub fn get_legal_movement_mask(&self, position: u8) -> u64 {
        let mut mask: u64 = 0;
        for legal_move in self.get_legal_moves(position) {
            let legal_move_parsed = ChessMove::unpack(legal_move);
            mask |= 1 << legal_move_parsed.destination;
        }
        mask
    }
    pub fn print_board(&self) {
        let mut text_representation = self.get_text_representation();
        text_representation.reverse();

        println!("----------");
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
        println!("\n----------");
    }

    pub fn is_in_checkmate(&self) -> bool {
        for possible_move in 0..64 {
            if !self.get_legal_moves(possible_move).is_empty() {
                return false;
            }
        }
        true
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

    pub fn move_piece(&self, chess_move: u16) -> Board {
        let chess_move = ChessMove::unpack(chess_move);
        let mut new_board = *self;
        let piece_type = new_board.find_piece(chess_move.position);

        new_board.bitboards[piece_type as usize].clear_bit(chess_move.position);

        match chess_move.move_type {
            MoveType::QuietMove => todo!(),
            MoveType::DoublePawnPush => {
                if chess_move.move_type == MoveType::DoublePawnPush {
                    new_board.en_passant = Some(chess_move.position);
                } else {
                    new_board.en_passant = None;
                }
                new_board.clear_square(chess_move.destination);
            }
            MoveType::KingCastle => todo!(),
            MoveType::QueenCastle => todo!(),
            MoveType::Capture => {
                new_board.clear_square(chess_move.destination);
            }
            MoveType::EnPassant => {
                let direction: i32 = match new_board.turn {
                    Color::White => -16,
                    Color::Black => 16,
                };
                new_board.clear_square((self.en_passant.unwrap() as i32 + direction) as u8);
            }
            MoveType::QueenPromotion => {
                new_board.clear_square(chess_move.destination);
                match self.turn {
                    Color::White => new_board.bitboards[Pieces::WhiteQueen as usize]
                        .set_bit(chess_move.destination),
                    Color::Black => new_board.bitboards[Pieces::BlackQueen as usize]
                        .set_bit(chess_move.destination),
                };
            }
            MoveType::RookPromotion => todo!(),
            MoveType::BishopPromotion => todo!(),
            MoveType::Knight => todo!(),
        }

        new_board.bitboards[piece_type as usize].set_bit(chess_move.destination);
        new_board.turn = new_board.switch_turn();

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

    pub fn get_pseudolegal_move_mask(&self, position: u8) -> u64 {
        let piece_type = self.find_piece(position);
        let piece_type_id = piece_type as usize;

        if piece_type == Pieces::None {
            // This is an empty square, no movement.
            return 0;
        }

        let piece_color = match piece_type_id / 6 {
            0..1 => Color::White,
            1..2 => Color::Black,
            _ => panic!(),
        };
        if piece_color != self.turn {
            return 0;
        }

        let lookup = [0, 1, 2, 3, 4, 9, 0, 1, 2, 3, 4, 9];

        let mut movement = self.lookup_tables[lookup[piece_type_id]][position as usize];

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
                movement |= self.lookup_tables[LookupTable::WhitePawnMoves as usize]
                    [position as usize]
                    & (UNIVERSE ^ occupancy);

                movement |= self.lookup_tables[LookupTable::WhitePawnCaptures as usize]
                    [position as usize]
                    & enemy_occupancy;
            }
            Pieces::BlackPawn => {
                movement |= self.lookup_tables[LookupTable::BlackPawnMoves as usize]
                    [position as usize]
                    & (UNIVERSE ^ occupancy);

                movement |= self.lookup_tables[LookupTable::BlackPawnCaptures as usize]
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
            movement &= self.raycast_calculate(position, occupancy);
        }

        movement &= UNIVERSE ^ friendly_occupancy;

        movement
    }

    fn get_pseudolegal_moves(&self, position: u8) -> Vec<u16> {
        let legal_move_mask = self.get_pseudolegal_move_mask(position);

        let mut move_buffer = Vec::new();
        for destination in 0..64 {
            let destination_mask = 1 << destination;
            if legal_move_mask & destination_mask != 0 {
                let piece = self.find_piece(position); // This will be refactored next pull request
                if let Some(en_passant_square) = self.en_passant {
                    match piece {
                        Pieces::WhitePawn => {
                            if self.lookup_tables[LookupTable::WhitePawnCaptures as usize]
                                [position as usize]
                                & (1 << (en_passant_square - 8))
                                != 0
                            {
                                move_buffer.push(ChessMove::pack(&ChessMove {
                                    position,
                                    destination: en_passant_square - 8,
                                    move_type: MoveType::EnPassant,
                                }));
                                continue;
                            }
                        }

                        Pieces::BlackPawn => {
                            if self.lookup_tables[LookupTable::BlackPawnCaptures as usize]
                                [position as usize]
                                & (1 << (en_passant_square + 8))
                                != 0
                            {
                                move_buffer.push(ChessMove::pack(&ChessMove {
                                    position,
                                    destination: en_passant_square + 8,
                                    move_type: MoveType::EnPassant,
                                }));
                                continue;
                            }
                        }

                        _ => {}
                    }
                }
                if piece == Pieces::WhitePawn || piece == Pieces::BlackPawn {
                    let ending_rank = match self.turn {
                        Color::White => 7,
                        Color::Black => 0,
                    };
                    let is_on_ending_rank = (destination / 8) == ending_rank;

                    if is_on_ending_rank {
                        move_buffer.push(ChessMove::pack(&ChessMove {
                            position,
                            destination,
                            move_type: MoveType::QueenPromotion,
                        }));
                        continue;
                    }
                    if (position as i32 - destination as i32).abs() > 8 {
                        move_buffer.push(ChessMove::pack(&ChessMove {
                            position,
                            destination,
                            move_type: MoveType::DoublePawnPush,
                        }));
                        continue;
                    } else {
                        move_buffer.push(ChessMove::pack(&ChessMove {
                            position,
                            destination,
                            move_type: MoveType::Capture,
                        }));
                        continue;
                    }
                }

                move_buffer.push(ChessMove::pack(&ChessMove {
                    position,
                    destination,
                    move_type: MoveType::Capture,
                }));
            }
        }

        move_buffer
    }

    pub fn try_make_move(&mut self, position: u8, destination: u8) {
        let legal_moves = self.get_legal_moves(position);
        for possible_move in legal_moves {
            if ChessMove::unpack(possible_move).destination == destination {
                *self = self.move_piece(possible_move);
            }
        }
    }

    fn find_kind_bitboard(&self, color: Color) -> BitBoard {
        match color {
            Color::White => self.bitboards[0],
            Color::Black => self.bitboards[6],
        }
    }

    pub fn get_legal_moves(&self, position: u8) -> Vec<u16> {
        let psuedo_legal_moves = self.get_pseudolegal_moves(position);

        let mut legal_move_buffer = Vec::new();
        for psuedo_legal_move in &psuedo_legal_moves {
            let chess_move = self.move_piece(*psuedo_legal_move);

            let king_bitmask = chess_move.find_kind_bitboard(chess_move.switch_turn());

            let mut enemy_bitmask = 0;
            for enemy_bit in 0..64 {
                let enemy_moves = chess_move.get_pseudolegal_moves(enemy_bit);
                for possible_response in enemy_moves {
                    let enemy_move_attack = ChessMove::unpack(possible_response).destination;
                    enemy_bitmask |= 1 << enemy_move_attack;
                }
            }

            if enemy_bitmask & king_bitmask.0 == 0 {
                legal_move_buffer.push(*psuedo_legal_move)
            }
        }

        legal_move_buffer
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
