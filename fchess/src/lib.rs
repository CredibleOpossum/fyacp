mod chess_data;

mod perft_tests;

use chess_data::generate_data;

pub mod structs;
use structs::*;

mod bitboard;
use bitboard::BitBoard;

mod magics;

mod constants;
use constants::*;

pub mod engine;

#[derive(Debug)]
struct Moves {
    move_buffer: [u16; MAX_LEGAL_MOVES],
    move_length: u8,
}

impl Default for Moves {
    fn default() -> Moves {
        Moves {
            move_buffer: [0u16; MAX_LEGAL_MOVES],
            move_length: 0,
        }
    }
}

#[derive(PartialEq)]
pub enum BoardState {
    Checkmate,
    Stalemate,
    OnGoing,
}

#[derive(Clone, Copy, Debug)]
struct CastlingRights {
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

pub struct ChessTables {
    lookup_tables: [[BitBoard; BOARD_SIZE]; 12],
}
impl Default for ChessTables {
    fn default() -> Self {
        ChessTables {
            lookup_tables: generate_data(),
        }
    }
}

#[derive(Clone)]
pub struct Board {
    pub bitboards: [[BitBoard; 6]; 2],
    castling_rights: CastlingRights,
    pub en_passant: Option<u8>, // Denotes the position of where the en passant square can be captured
    pub turn: Color,
}

impl Default for Board {
    fn default() -> Self {
        Board {
            bitboards: STARTING_POSITION,
            castling_rights: CastlingRights::default(),
            en_passant: None,
            turn: Color::White,
        }
    }
}

fn bishop_moves(position: u8, occupancy: BitBoard, tables: &ChessTables) -> BitBoard {
    let movement_mask = tables.lookup_tables[LookupTable::BishopMoves as usize][position as usize]; // Short rook bitmask
    let key = ((movement_mask & occupancy) * BitBoard(magics::MAGICS_BISHOP[position as usize])).0
        >> magics::MAGIC_SHIFT_BISHOP;
    BitBoard(magics::LOOKUP_BISHOP[position as usize][key as usize])
}

fn rook_moves(position: u8, occupancy: BitBoard, tables: &ChessTables) -> BitBoard {
    let movement_mask = tables.lookup_tables[LookupTable::RookMoves as usize][position as usize]; // Short rook bitmask
    let key = ((movement_mask & occupancy) * BitBoard(magics::MAGICS_ROOK[position as usize])).0
        >> magics::MAGIC_SHIFT_ROOK;
    BitBoard(magics::LOOKUP_ROOK[position as usize][key as usize])
}

impl Board {
    fn get_white_occupancy(&self) -> BitBoard {
        let bitboards: [BitBoard; 6] = self.bitboards[Color::White as usize];
        bitboards[0] | bitboards[1] | bitboards[2] | bitboards[3] | bitboards[4] | bitboards[5]
    }

    fn get_black_occupancy(&self) -> BitBoard {
        let bitboards: [BitBoard; 6] = self.bitboards[Color::Black as usize];
        bitboards[0] | bitboards[1] | bitboards[2] | bitboards[3] | bitboards[4] | bitboards[5]
    }

    pub fn get_legal_movement_mask(&self, position: u8, tables: &ChessTables) -> u64 {
        let mut mask: u64 = 0;
        let moves = self.get_legal_moves(position, tables);
        for legal_move in 0..moves.move_length {
            let legal_move_parsed = ChessMove::unpack(moves.move_buffer[legal_move as usize]);
            mask |= 1 << legal_move_parsed.destination;
        }
        mask
    }

    fn get_full_capture_mask(&self, color: Color, tables: &ChessTables) -> BitBoard {
        let mut board_capturemask = BitBoard(0);

        let mut occupancy = match color {
            Color::White => self.get_white_occupancy(),
            Color::Black => self.get_black_occupancy(),
        };

        while !occupancy.is_empty() {
            let index = occupancy.get_index_and_pop();
            board_capturemask |= self.get_pseudolegal_capture_mask(index, color, tables).0;
        }

        board_capturemask
    }

    fn is_in_check(&self, tables: &ChessTables) -> bool {
        let enemy_bitmask = self.get_full_capture_mask(self.turn.opposite(), tables);

        !(self.find_kind_bitboard(self.turn) & enemy_bitmask).is_empty()
    }

    pub fn get_board_state(&self, tables: &ChessTables) -> BoardState {
        let mut self_occupancy = match self.turn {
            Color::White => self.get_white_occupancy(),
            Color::Black => self.get_black_occupancy(),
        };

        while !self_occupancy.is_empty() {
            let index = self_occupancy.get_index_and_pop();
            if self.get_legal_moves(index, tables).move_length != 0 {
                return BoardState::OnGoing;
            }
        }

        if self.is_in_check(tables) {
            return BoardState::Checkmate;
        }

        BoardState::Stalemate
    }

    fn clear_square(&mut self, position: u8, color: Color) {
        for bitboard in &mut self.bitboards[color as usize] {
            bitboard.clear_bit(position);
        }
    }

    fn find_piece(&self, position: u8) -> (Pieces, Color) {
        for index in 0..6 {
            if self.bitboards[Color::White as usize][index].get_bit(position) {
                return (Pieces::from_u8(index as u8), Color::White);
            }
            if self.bitboards[Color::Black as usize][index].get_bit(position) {
                return (Pieces::from_u8(index as u8), Color::Black);
            }
        }
        (Pieces::None, Color::White) // This should be a panic in the future
    }

    pub fn other_color(&self) -> Color {
        match self.turn {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }

    pub fn move_piece(&self, chess_move: u16) -> Board {
        let chess_move = ChessMove::unpack(chess_move);

        let mut new_board = self.clone();
        let (piece_type, color) = new_board.find_piece(chess_move.origin);
        let color_index = color as usize;

        new_board.bitboards[color_index][piece_type as usize].clear_bit(chess_move.origin);

        match chess_move.move_type {
            MoveType::QuietMove => todo!(),
            MoveType::DoublePawnPush => {
                new_board.bitboards[color_index][piece_type as usize]
                    .set_bit(chess_move.destination);
            }
            MoveType::KingCastle => {
                new_board.bitboards[color_index][piece_type as usize]
                    .set_bit(chess_move.destination);

                new_board.bitboards[color_index][Pieces::Rook as usize]
                    .clear_bit(chess_move.destination - 1);
                new_board.bitboards[color_index][Pieces::Rook as usize]
                    .set_bit(chess_move.destination + 1);
            }
            MoveType::QueenCastle => {
                new_board.bitboards[color_index][piece_type as usize]
                    .set_bit(chess_move.destination);

                new_board.bitboards[color_index][Pieces::Rook as usize]
                    .clear_bit(chess_move.destination + 2);
                new_board.bitboards[color_index][Pieces::Rook as usize]
                    .set_bit(chess_move.destination - 1);
            }
            MoveType::Capture => {
                new_board.clear_square(chess_move.destination, color.opposite());
                new_board.bitboards[color_index][piece_type as usize]
                    .set_bit(chess_move.destination);
            }
            MoveType::EnPassant => {
                let direction: i32 = match new_board.turn {
                    Color::White => -16,
                    Color::Black => 16,
                };
                new_board.clear_square(
                    (self.en_passant.unwrap() as i32 + direction) as u8,
                    color.opposite(),
                );
                new_board.bitboards[color_index][piece_type as usize]
                    .set_bit(chess_move.destination);
            }
            MoveType::QueenPromotion => {
                new_board.clear_square(chess_move.destination, color.opposite());
                match self.turn {
                    Color::White => new_board.bitboards[color_index][Pieces::Queen as usize]
                        .set_bit(chess_move.destination),
                    Color::Black => new_board.bitboards[color_index][Pieces::Queen as usize]
                        .set_bit(chess_move.destination),
                };
            }
            MoveType::RookPromotion => {
                new_board.clear_square(chess_move.destination, color.opposite());
                match self.turn {
                    Color::White => new_board.bitboards[color_index][Pieces::Rook as usize]
                        .set_bit(chess_move.destination),
                    Color::Black => new_board.bitboards[color_index][Pieces::Rook as usize]
                        .set_bit(chess_move.destination),
                };
            }
            MoveType::BishopPromotion => {
                new_board.clear_square(chess_move.destination, color.opposite());
                match self.turn {
                    Color::White => new_board.bitboards[color_index][Pieces::Bishop as usize]
                        .set_bit(chess_move.destination),
                    Color::Black => new_board.bitboards[color_index][Pieces::Bishop as usize]
                        .set_bit(chess_move.destination),
                };
            }
            MoveType::KnightPromotion => {
                new_board.clear_square(chess_move.destination, color.opposite());
                match self.turn {
                    Color::White => new_board.bitboards[color_index][Pieces::Knight as usize]
                        .set_bit(chess_move.destination),
                    Color::Black => new_board.bitboards[color_index][Pieces::Knight as usize]
                        .set_bit(chess_move.destination),
                };
            }
        }

        if chess_move.move_type == MoveType::DoublePawnPush {
            new_board.en_passant = Some(chess_move.origin);
        } else {
            new_board.en_passant = None;
        }

        if piece_type == Pieces::King {
            match color {
                Color::White => {
                    new_board.castling_rights.white_kingside = false;
                    new_board.castling_rights.white_queenside = false;
                }

                Color::Black => {
                    new_board.castling_rights.black_kingside = false;
                    new_board.castling_rights.black_queenside = false;
                }
            }
        }

        // Lets handle rook castling rights, if a rook is moved we need to get rid of castling for that rook.
        // It doesn't matter if a rook is being moved since the castling rights would already be gone.
        match chess_move.origin {
            0 => new_board.castling_rights.white_kingside = false,
            7 => new_board.castling_rights.white_queenside = false,

            63 => new_board.castling_rights.black_queenside = false,
            56 => new_board.castling_rights.black_kingside = false,
            _ => {}
        }

        // Same as before
        match chess_move.destination {
            0 => new_board.castling_rights.white_kingside = false,
            7 => new_board.castling_rights.white_queenside = false,

            63 => new_board.castling_rights.black_queenside = false,
            56 => new_board.castling_rights.black_kingside = false,
            _ => {}
        }

        new_board.turn = new_board.other_color();

        new_board
    }

    pub fn get_pseudolegal_capture_mask(
        &self,
        position: u8,
        color: Color,
        tables: &ChessTables,
    ) -> (BitBoard, BitBoard, BitBoard) {
        let (piece_type, _) = self.find_piece(position);

        let mut friendly_occupancy = self.get_white_occupancy();
        let mut enemy_occupancy = self.get_black_occupancy();
        if color != Color::White {
            std::mem::swap(&mut friendly_occupancy, &mut enemy_occupancy);
        }
        let occupancy = friendly_occupancy | enemy_occupancy;

        let mut movement_mask = BitBoard(0);
        match piece_type {
            Pieces::King => {
                movement_mask =
                    tables.lookup_tables[LookupTable::KingMoves as usize][position as usize];
                movement_mask &= !friendly_occupancy;
            }
            Pieces::Pawn => match color {
                Color::White => {
                    movement_mask |= tables.lookup_tables[LookupTable::WhitePawnMoves as usize]
                        [position as usize]
                        & !occupancy;

                    if (tables.lookup_tables[LookupTable::WhitePawnMoves as usize]
                        [position as usize]
                        & occupancy)
                        .is_empty()
                    {
                        movement_mask |= tables.lookup_tables
                            [LookupTable::WhitePawnLongMoves as usize]
                            [position as usize]
                            & !occupancy;
                    }
                    movement_mask |= tables.lookup_tables[LookupTable::WhitePawnCaptures as usize]
                        [position as usize]
                }

                Color::Black => {
                    movement_mask |= tables.lookup_tables[LookupTable::BlackPawnMoves as usize]
                        [position as usize]
                        & !occupancy;

                    if (tables.lookup_tables[LookupTable::BlackPawnMoves as usize]
                        [position as usize]
                        & occupancy)
                        .is_empty()
                    {
                        movement_mask |= tables.lookup_tables
                            [LookupTable::BlackPawnLongMoves as usize]
                            [position as usize]
                            & !occupancy;
                    }
                    movement_mask |= tables.lookup_tables[LookupTable::BlackPawnCaptures as usize]
                        [position as usize]
                        & enemy_occupancy;
                }
            },

            Pieces::Knight => {
                movement_mask =
                    tables.lookup_tables[LookupTable::KnightMoves as usize][position as usize];
                movement_mask &= !friendly_occupancy;
            }

            Pieces::Rook => {
                movement_mask = rook_moves(position, occupancy, tables);
                movement_mask &= !friendly_occupancy;
            }
            Pieces::Bishop => {
                movement_mask = bishop_moves(position, occupancy, tables);
                movement_mask &= !friendly_occupancy;
            }
            Pieces::Queen => {
                movement_mask = rook_moves(position, occupancy, tables)
                    | bishop_moves(position, occupancy, tables);
                movement_mask &= !friendly_occupancy;
            }
            Pieces::None => panic!(),
        }
        (movement_mask, friendly_occupancy, enemy_occupancy)
    }

    fn get_pseudolegal_moves(&self, position: u8, tables: &ChessTables) -> Moves {
        let (mut psuedolegal_capture_mask, friendly_occupacny, enemy_occupancy) =
            self.get_pseudolegal_capture_mask(position, self.turn, tables);

        let mut move_buffer = Moves::default();
        let mut move_position = 0;

        let (piece, color) = self.find_piece(position);
        if piece == Pieces::None {
            return Moves::default();
        }
        if piece == Pieces::Pawn && self.en_passant.is_some() {
            match color {
                Color::White => {
                    if !(tables.lookup_tables[LookupTable::WhitePawnCaptures as usize]
                        [position as usize]
                        & BitBoard(1 << (self.en_passant.unwrap() - 8)))
                    .is_empty()
                    {
                        move_buffer.move_buffer[move_position] = ChessMove::pack(&ChessMove {
                            origin: position,
                            destination: self.en_passant.unwrap() - 8,
                            move_type: MoveType::EnPassant,
                        });
                        move_position += 1;
                    }
                }
                Color::Black => {
                    if !(tables.lookup_tables[LookupTable::BlackPawnCaptures as usize]
                        [position as usize]
                        & BitBoard(1 << (self.en_passant.unwrap() + 8)))
                    .is_empty()
                    {
                        move_buffer.move_buffer[move_position] = ChessMove::pack(&ChessMove {
                            origin: position,
                            destination: self.en_passant.unwrap() + 8,
                            move_type: MoveType::EnPassant,
                        });
                        move_position += 1;
                    }
                }
            }
        }

        while !psuedolegal_capture_mask.is_empty() {
            let destination = psuedolegal_capture_mask.get_index_and_pop();

            let (piece, color) = self.find_piece(position);

            let is_long_move = match color {
                Color::White => !(tables.lookup_tables[LookupTable::WhitePawnLongMoves as usize]
                    [position as usize]
                    & BitBoard(1 << destination))
                .is_empty(),
                Color::Black => !(tables.lookup_tables[LookupTable::BlackPawnLongMoves as usize]
                    [position as usize]
                    & BitBoard(1 << destination))
                .is_empty(),
            };

            if piece == Pieces::Pawn && is_long_move {
                move_buffer.move_buffer[move_position] = ChessMove::pack(&ChessMove {
                    origin: position,
                    destination,
                    move_type: MoveType::DoublePawnPush,
                });
                move_position += 1;
                continue;
            }

            if piece == Pieces::Pawn {
                let is_pawn_capture = (position % 8) != (destination % 8);
                if is_pawn_capture && ((BitBoard(1 << destination) & enemy_occupancy).is_empty()) {
                    continue; // This isn't a legal pawn move, since it's capturing but not hitting an enemy.
                }
                let last_rank = match color {
                    Color::White => 7,
                    Color::Black => 0,
                };
                let is_moving_to_last_rank =
                    piece == Pieces::Pawn && ((destination / 8) == last_rank);
                if is_moving_to_last_rank {
                    move_buffer.move_buffer[move_position] = ChessMove::pack(&ChessMove {
                        origin: position,
                        destination,
                        move_type: MoveType::QueenPromotion,
                    });
                    move_position += 1;
                    move_buffer.move_buffer[move_position] = ChessMove::pack(&ChessMove {
                        origin: position,
                        destination,
                        move_type: MoveType::RookPromotion,
                    });
                    move_position += 1;
                    move_buffer.move_buffer[move_position] = ChessMove::pack(&ChessMove {
                        origin: position,
                        destination,
                        move_type: MoveType::BishopPromotion,
                    });
                    move_position += 1;
                    move_buffer.move_buffer[move_position] = ChessMove::pack(&ChessMove {
                        origin: position,
                        destination,
                        move_type: MoveType::KnightPromotion,
                    });
                    move_position += 1;
                    continue;
                }
            }

            move_buffer.move_buffer[move_position] = ChessMove::pack(&ChessMove {
                origin: position,
                destination,
                move_type: MoveType::Capture,
            });
            move_position += 1;
        }

        // CASTLING

        let enemy_hitmask = self.get_full_capture_mask(self.other_color(), tables);

        let blocking_pieces = enemy_occupancy | friendly_occupacny;

        let white_king_location_x = 3; // Assuming we have castling rights, we know the position of both the king and rook.
        let black_king_location_x = 3 + (7 * 8);

        // This is kinda ugly
        let white_kingside_hitmask = BitBoard(0xe);
        let white_queenside_hitmask = BitBoard(0x38);
        let black_kingside_hitmask = BitBoard(0xe00000000000000);
        let black_queenside_hitmask = BitBoard(0x3800000000000000);

        let white_kingside_hitmask_friendly = BitBoard(0x6);
        let white_queenside_hitmask_friendly = BitBoard(0x70);
        let black_kingside_hitmask_friendly = BitBoard(0x600000000000000);
        let black_queenside_hitmask_friendly = BitBoard(0x7000000000000000);

        if piece == Pieces::King {
            match color {
                Color::White => {
                    if self.turn == Color::White {
                        if self.castling_rights.white_kingside
                            && (enemy_hitmask & white_kingside_hitmask).is_empty()
                            && (blocking_pieces & white_kingside_hitmask_friendly).is_empty()
                        {
                            move_buffer.move_buffer[move_position] = ChessMove::pack(&ChessMove {
                                origin: white_king_location_x,
                                destination: white_king_location_x - 2,
                                move_type: MoveType::KingCastle,
                            });
                            move_position += 1;
                        }
                        if self.castling_rights.white_queenside
                            && (enemy_hitmask & white_queenside_hitmask).is_empty()
                            && (blocking_pieces & white_queenside_hitmask_friendly).is_empty()
                        {
                            move_buffer.move_buffer[move_position] = ChessMove::pack(&ChessMove {
                                origin: white_king_location_x,
                                destination: white_king_location_x + 2,
                                move_type: MoveType::QueenCastle,
                            });
                            move_position += 1;
                        }
                    }
                }
                Color::Black => {
                    if self.turn == Color::Black {
                        if self.castling_rights.black_kingside
                            && (enemy_hitmask & black_kingside_hitmask).is_empty()
                            && (blocking_pieces & black_kingside_hitmask_friendly).is_empty()
                        {
                            move_buffer.move_buffer[move_position] = ChessMove::pack(&ChessMove {
                                origin: black_king_location_x,
                                destination: black_king_location_x - 2,
                                move_type: MoveType::KingCastle,
                            });
                            move_position += 1;
                        }

                        if self.castling_rights.black_queenside
                            && (enemy_hitmask & black_queenside_hitmask).is_empty()
                            && (blocking_pieces & black_queenside_hitmask_friendly).is_empty()
                        {
                            move_buffer.move_buffer[move_position] = ChessMove::pack(&ChessMove {
                                origin: black_king_location_x,
                                destination: black_king_location_x + 2,
                                move_type: MoveType::QueenCastle,
                            });
                            move_position += 1;
                        }
                    }
                }
            }
        }
        move_buffer.move_length = move_position as u8;
        move_buffer
    }

    pub fn try_make_move(&mut self, position: u8, destination: u8, tables: &ChessTables) {
        let legal_moves = self.get_all_legal_moves(tables);
        for possible_move in 0..legal_moves.1 {
            let parsed_move = ChessMove::unpack(legal_moves.0[possible_move]);
            match parsed_move.move_type {
                MoveType::QueenPromotion => {}
                MoveType::RookPromotion => continue,
                MoveType::BishopPromotion => continue,
                MoveType::KnightPromotion => continue,

                _ => {}
            }
            if parsed_move.origin == position && parsed_move.destination == destination {
                *self = self.move_piece(legal_moves.0[possible_move]);
            }
        }
    }

    pub fn get_legal_movement_mask_safe(&self, position: u8, tables: &ChessTables) -> BitBoard {
        // This function exists because the regular one has no safety and will crash if the input isn't ideal
        let (piece_type, color) = self.find_piece(position);
        if self.turn != color || piece_type == Pieces::None {
            BitBoard(0)
        } else {
            BitBoard(self.get_legal_movement_mask(position, tables))
        }
    }

    fn find_kind_bitboard(&self, color: Color) -> BitBoard {
        self.bitboards[color as usize][0]
    }

    fn get_legal_moves(&self, position: u8, tables: &ChessTables) -> Moves {
        let psuedo_legal_moves = self.get_pseudolegal_moves(position, tables);
        let mut legal_move_buffer = Moves::default();
        let mut legal_move_index = 0;

        for psuedo_legal_move_index in 0..psuedo_legal_moves.move_length {
            let chess_move =
                self.move_piece(psuedo_legal_moves.move_buffer[psuedo_legal_move_index as usize]);

            let king_bitmask = chess_move.find_kind_bitboard(chess_move.turn.opposite());

            let mut enemy_bitmask = BitBoard(0);
            let mut enemy_occupancy = match chess_move.turn {
                Color::White => chess_move.get_white_occupancy(),
                Color::Black => chess_move.get_black_occupancy(),
            };

            while !enemy_occupancy.is_empty() {
                let index = enemy_occupancy.get_index_and_pop();
                enemy_bitmask |= chess_move
                    .get_pseudolegal_capture_mask(index, chess_move.turn, tables)
                    .0;
            }

            if (enemy_bitmask & king_bitmask).is_empty() {
                legal_move_buffer.move_buffer[legal_move_index] =
                    psuedo_legal_moves.move_buffer[psuedo_legal_move_index as usize];
                legal_move_index += 1;
            }
        }

        legal_move_buffer.move_length = legal_move_index as u8;
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

        let characters_white = ["♚", "♛", "♜", "♝", "♞", "♟"]; // Sorted by material value
        let characters_black = ["♔", "♕", "♖", "♗", "♘", "♙"];
        for index in 0..characters_white.len() {
            insert_chess_pieces(
                self.bitboards[0][index],
                characters_white[index],
                &mut text_representation,
            );
            insert_chess_pieces(
                self.bitboards[1][index],
                characters_black[index],
                &mut text_representation,
            );
        }

        text_representation
    }

    fn get_all_legal_moves(&self, tables: &ChessTables) -> ([u16; MAX_MOVE_BUFFER], usize) {
        let mut move_buffer = [0; MAX_MOVE_BUFFER];
        let mut array_position: usize = 0;

        let mut self_occupancy = match self.turn {
            Color::White => self.get_white_occupancy(),
            Color::Black => self.get_black_occupancy(),
        };

        while !self_occupancy.is_empty() {
            let index = self_occupancy.get_index_and_pop();
            let moves = self.get_legal_moves(index, tables);

            move_buffer[array_position..array_position + moves.move_length as usize]
                .clone_from_slice(&moves.move_buffer[0..moves.move_length as usize]);
            array_position += moves.move_length as usize;
        }

        (move_buffer, array_position)
    }
    pub fn fen_parser(fen: &str) -> Board {
        // Doesn't parse en_passant square.
        let mut board = Board::default();

        let mut index: usize = 0;
        let split_fen: Vec<&str> = fen.split(' ').collect();

        for bitboard in 0..board.bitboards.len() {
            board.bitboards[bitboard] = [BitBoard(0); 6];
        }

        for character in split_fen[0].chars().rev() {
            if !split_fen[2].contains('Q') {
                board.castling_rights.white_queenside = false;
            }
            if !split_fen[2].contains('K') {
                board.castling_rights.white_kingside = false;
            }
            if !split_fen[2].contains('q') {
                board.castling_rights.black_queenside = false;
            }
            if !split_fen[2].contains('k') {
                board.castling_rights.black_kingside = false;
            }

            match character {
                'P' => board.bitboards[0][Pieces::Pawn as usize].0 |= 1 << index,
                'p' => board.bitboards[1][Pieces::Pawn as usize].0 |= 1 << index,

                'N' => board.bitboards[0][Pieces::Knight as usize].0 |= 1 << index,
                'n' => board.bitboards[1][Pieces::Knight as usize].0 |= 1 << index,

                'B' => board.bitboards[0][Pieces::Bishop as usize].0 |= 1 << index,
                'b' => board.bitboards[1][Pieces::Bishop as usize].0 |= 1 << index,

                'R' => board.bitboards[0][Pieces::Rook as usize].0 |= 1 << index,
                'r' => board.bitboards[1][Pieces::Rook as usize].0 |= 1 << index,

                'Q' => board.bitboards[0][Pieces::Queen as usize].0 |= 1 << index,
                'q' => board.bitboards[1][Pieces::Queen as usize].0 |= 1 << index,

                'K' => board.bitboards[0][Pieces::King as usize].0 |= 1 << index,
                'k' => board.bitboards[1][Pieces::King as usize].0 |= 1 << index,

                '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' => {
                    index += character.to_string().parse::<i32>().unwrap() as usize;
                    continue;
                }
                _ => {
                    continue;
                }
            }

            index += 1;
            if index >= 64 {
                break;
            }
        }

        let turn = match split_fen[1] {
            "w" => Color::White,
            "b" => Color::Black,
            _ => panic!("Invaild fen, incorrect turn?"),
        };

        board.turn = turn;

        board
    }
}

pub fn human_readable_position(position: u8) -> String {
    HUMAN_READBLE_SQAURES[position as usize].to_string()
}
