use crate::{
    bitboard::BitBoard, magics, Board, BoardState, ChessMove, ChessTables, Color, LookupTable,
    MoveType, Moves, Pieces, EMPTY_STRING, HUMAN_READBLE_SQAURES, MAX_MOVE_BUFFER,
};

use crate::constants::*;

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

    pub fn get_full_capture_mask(&self, color: Color, tables: &ChessTables) -> BitBoard {
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
        let legal_moves = self.get_all_legal_moves(tables);
        if legal_moves.1 != 0 {
            return BoardState::OnGoing;
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
            MoveType::QuietMove => new_board.bitboards[color_index][piece_type as usize]
                .set_bit(chess_move.destination),
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
                    Color::White => -8,
                    Color::Black => 8,
                };
                let en_pasant_location = (chess_move.destination as i32 + direction) as u8;
                new_board.clear_square(en_pasant_location, color.opposite());
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
            match new_board.turn {
                Color::White => new_board.en_passant = Some(chess_move.origin + 8),
                Color::Black => new_board.en_passant = Some(chess_move.origin - 8),
            }
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

        let movement_mask = match piece_type {
            Pieces::King => generate_king_bitmask(tables, friendly_occupancy, position),
            Pieces::Pawn => {
                generate_pawn_bitmask(color, tables, position, occupancy, enemy_occupancy)
            }
            Pieces::Knight => generate_knight_bitmask(tables, position, friendly_occupancy),
            Pieces::Rook => generate_rook_bitmask(position, occupancy, tables, friendly_occupancy),
            Pieces::Bishop => {
                generate_bishop_bitmask(position, occupancy, tables, friendly_occupancy)
            }
            Pieces::Queen => {
                generate_queen_bitmask(position, occupancy, tables, friendly_occupancy)
            }
            Pieces::None => panic!(),
        };

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
                        & BitBoard(1 << (self.en_passant.unwrap())))
                    .is_empty()
                    {
                        move_buffer.move_buffer[move_position] = ChessMove::pack(&ChessMove {
                            origin: position,
                            destination: self.en_passant.unwrap(),
                            move_type: MoveType::EnPassant,
                        });
                        move_position += 1;
                    }
                }
                Color::Black => {
                    if !(tables.lookup_tables[LookupTable::BlackPawnCaptures as usize]
                        [position as usize]
                        & BitBoard(1 << (self.en_passant.unwrap())))
                    .is_empty()
                    {
                        move_buffer.move_buffer[move_position] = ChessMove::pack(&ChessMove {
                            origin: position,
                            destination: self.en_passant.unwrap(),
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

            match !(BitBoard(1 << destination) & enemy_occupancy).is_empty() {
                true => {
                    move_buffer.move_buffer[move_position] = ChessMove::pack(&ChessMove {
                        origin: position,
                        destination,
                        move_type: MoveType::Capture,
                    });
                }
                false => {
                    move_buffer.move_buffer[move_position] = ChessMove::pack(&ChessMove {
                        origin: position,
                        destination,
                        move_type: MoveType::QuietMove,
                    });
                }
            }

            move_position += 1;
        }

        // CASTLING

        let enemy_hitmask = self.get_full_capture_mask(self.other_color(), tables);

        let blocking_pieces = enemy_occupancy | friendly_occupacny;

        let white_king_location_x = 3; // Assuming we have castling rights, we know the position of both the king and rook.
        let black_king_location_x = 3 + (7 * 8);

        if piece == Pieces::King {
            match color {
                Color::White => {
                    if self.turn == Color::White {
                        if self.castling_rights.white_kingside
                            && (enemy_hitmask & WHITE_KINGSIDE_HITMASK).is_empty()
                            && (blocking_pieces & WHITE_KINGSIDE_HITMASK_BLOCKERS).is_empty()
                        {
                            move_buffer.move_buffer[move_position] = ChessMove::pack(&ChessMove {
                                origin: white_king_location_x,
                                destination: white_king_location_x - 2,
                                move_type: MoveType::KingCastle,
                            });
                            move_position += 1;
                        }
                        if self.castling_rights.white_queenside
                            && (enemy_hitmask & WHITE_QUEENSIDE_HITMASK).is_empty()
                            && (blocking_pieces & WHITE_QUEENSIDE_HITMASK_BLOCKERS).is_empty()
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
                            && (enemy_hitmask & BLACK_KINGSIDE_HITMASK).is_empty()
                            && (blocking_pieces & BLACK_KINGSIDE_HITMASK_BLOCKERS).is_empty()
                        {
                            move_buffer.move_buffer[move_position] = ChessMove::pack(&ChessMove {
                                origin: black_king_location_x,
                                destination: black_king_location_x - 2,
                                move_type: MoveType::KingCastle,
                            });
                            move_position += 1;
                        }

                        if self.castling_rights.black_queenside
                            && (enemy_hitmask & BLACK_QUEENSIDE_HITMASK).is_empty()
                            && (blocking_pieces & BLACK_QUEENSIDE_HITMASK_BLOCKERS).is_empty()
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

    pub fn try_make_move(
        &mut self,
        position: u8,
        destination: u8,
        promotion_preference: char,
        tables: &ChessTables,
    ) {
        let legal_moves = self.get_all_legal_moves(tables);
        for possible_move in 0..legal_moves.1 {
            let parsed_move = ChessMove::unpack(legal_moves.0[possible_move]);
            match parsed_move.move_type {
                MoveType::QueenPromotion => {
                    if promotion_preference != 'q' {
                        continue;
                    }
                }
                MoveType::RookPromotion => {
                    if promotion_preference != 'r' {
                        continue;
                    }
                }
                MoveType::BishopPromotion => {
                    if promotion_preference != 'b' {
                        continue;
                    }
                }
                MoveType::KnightPromotion => {
                    if promotion_preference != 'k' {
                        continue;
                    }
                }

                _ => {}
            }
            if parsed_move.origin == position && parsed_move.destination == destination {
                *self = self.move_piece(legal_moves.0[possible_move]);
            }
        }
    }

    fn find_kind_bitboard(&self, color: Color) -> BitBoard {
        self.bitboards[color as usize][0]
    }

    /*
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
    */

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

    pub fn get_all_legal_moves(&self, tables: &ChessTables) -> ([u16; MAX_MOVE_BUFFER], usize) {
        let mut psuedolegal_move_buffer = [0; MAX_MOVE_BUFFER];
        let mut array_position: usize = 0;

        let mut legal_move_buffer = [0; MAX_MOVE_BUFFER];
        let mut legal_move_position = 0;

        let mut self_occupancy = match self.turn {
            Color::White => self.get_white_occupancy(),
            Color::Black => self.get_black_occupancy(),
        };

        while !self_occupancy.is_empty() {
            let index = self_occupancy.get_index_and_pop();
            let moves = self.get_pseudolegal_moves(index, tables);

            psuedolegal_move_buffer[array_position..array_position + moves.move_length as usize]
                .clone_from_slice(&moves.move_buffer[0..moves.move_length as usize]);
            array_position += moves.move_length as usize;
        }

        // Now we have a buffer of all psuedolegal moves
        for move_index in 0..array_position {
            let chess_move = psuedolegal_move_buffer[move_index];
            let temp_board = self.move_piece(chess_move);

            let mut friendly_occupancy = temp_board.get_white_occupancy();
            let mut enemy_occupancy = temp_board.get_black_occupancy();
            let mut occupancy = friendly_occupancy | enemy_occupancy;
            if self.turn != Color::White {
                std::mem::swap(&mut friendly_occupancy, &mut enemy_occupancy);
            }

            let enemy_bitboards = match self.turn {
                Color::White => temp_board.bitboards[1],
                Color::Black => temp_board.bitboards[0],
            };

            let king_bitmask = temp_board.find_kind_bitboard(self.turn); // Find king bitboard of who just turned, for preventing moving into check.
            let king_position = king_bitmask.0.trailing_zeros() as u8;
            friendly_occupancy.0 ^= 1 << king_position; // Remove king from bitmask so it can be hit.
            occupancy.0 ^= 1 << king_position;

            if king_position == 64 {
                continue;
            }

            let knight_inverse = generate_knight_bitmask(tables, king_position, friendly_occupancy)
                & enemy_bitboards[Pieces::Knight as usize];
            if !knight_inverse.is_empty() {
                // Piece is giving check
                continue; // Move onto next psuedolegal move
            }

            let king_inverse = generate_king_bitmask(tables, friendly_occupancy, king_position)
                & enemy_bitboards[Pieces::King as usize];
            if !king_inverse.is_empty() {
                continue;
            }

            let pawn_inverse =
                generate_pawn_bitmask(self.turn, tables, king_position, occupancy, enemy_occupancy)
                    & enemy_bitboards[Pieces::Pawn as usize];
            if !pawn_inverse.is_empty() {
                continue;
            }

            let bishop_inverse =
                generate_bishop_bitmask(king_position, occupancy, tables, friendly_occupancy)
                    & enemy_bitboards[Pieces::Bishop as usize];
            if !bishop_inverse.is_empty() {
                continue;
            }

            let queen_inverse =
                generate_queen_bitmask(king_position, occupancy, tables, friendly_occupancy)
                    & enemy_bitboards[Pieces::Queen as usize];
            if !queen_inverse.is_empty() {
                continue;
            }

            let rook_inverse =
                generate_rook_bitmask(king_position, occupancy, tables, friendly_occupancy)
                    & enemy_bitboards[Pieces::Rook as usize];
            if !rook_inverse.is_empty() {
                continue;
            }

            // All legal checks passed
            legal_move_buffer[legal_move_position] = chess_move;
            legal_move_position += 1;
        }

        (legal_move_buffer, legal_move_position)
    }
}

fn generate_queen_bitmask(
    position: u8,
    occupancy: BitBoard,
    tables: &ChessTables,
    friendly_occupancy: BitBoard,
) -> BitBoard {
    let mut movement_mask =
        rook_moves(position, occupancy, tables) | bishop_moves(position, occupancy, tables);
    movement_mask &= !friendly_occupancy;
    movement_mask
}

fn generate_bishop_bitmask(
    position: u8,
    occupancy: BitBoard,
    tables: &ChessTables,
    friendly_occupancy: BitBoard,
) -> BitBoard {
    let mut movement_mask = bishop_moves(position, occupancy, tables);
    movement_mask &= !friendly_occupancy;
    movement_mask
}

fn generate_rook_bitmask(
    position: u8,
    occupancy: BitBoard,
    tables: &ChessTables,
    friendly_occupancy: BitBoard,
) -> BitBoard {
    let mut movement_mask = rook_moves(position, occupancy, tables);
    movement_mask &= !friendly_occupancy;
    movement_mask
}

fn generate_knight_bitmask(
    tables: &ChessTables,
    position: u8,
    friendly_occupancy: BitBoard,
) -> BitBoard {
    let mut movement_mask =
        tables.lookup_tables[LookupTable::KnightMoves as usize][position as usize];
    movement_mask &= !friendly_occupancy;
    movement_mask
}

fn generate_pawn_bitmask(
    color: Color,
    tables: &ChessTables,
    position: u8,
    occupancy: BitBoard,
    enemy_occupancy: BitBoard,
) -> BitBoard {
    let mut movement_mask = BitBoard(0);
    match color {
        Color::White => {
            movement_mask |= tables.lookup_tables[LookupTable::WhitePawnMoves as usize]
                [position as usize]
                & !occupancy;

            if (tables.lookup_tables[LookupTable::WhitePawnMoves as usize][position as usize]
                & occupancy)
                .is_empty()
            {
                movement_mask |= tables.lookup_tables[LookupTable::WhitePawnLongMoves as usize]
                    [position as usize]
                    & !occupancy;
            }
            movement_mask |=
                tables.lookup_tables[LookupTable::WhitePawnCaptures as usize][position as usize]
        }

        Color::Black => {
            movement_mask |= tables.lookup_tables[LookupTable::BlackPawnMoves as usize]
                [position as usize]
                & !occupancy;

            if (tables.lookup_tables[LookupTable::BlackPawnMoves as usize][position as usize]
                & occupancy)
                .is_empty()
            {
                movement_mask |= tables.lookup_tables[LookupTable::BlackPawnLongMoves as usize]
                    [position as usize]
                    & !occupancy;
            }
            movement_mask |= tables.lookup_tables[LookupTable::BlackPawnCaptures as usize]
                [position as usize]
                & enemy_occupancy;
        }
    }
    movement_mask
}

pub fn human_readable_position(position: u8) -> String {
    HUMAN_READBLE_SQAURES[position as usize].to_string()
}

fn perft_internal(board: Board, depth: u8, max_depth: u8, tables: &ChessTables) -> usize {
    let all_legal_moves = board.get_all_legal_moves(tables);
    if depth == max_depth {
        return all_legal_moves.1;
    }

    match board.get_board_state(tables) {
        BoardState::Checkmate => return all_legal_moves.1,
        BoardState::Stalemate => return all_legal_moves.1,
        BoardState::OnGoing => {}
    }

    let mut move_sum = 0;

    for possible_move in 0..all_legal_moves.1 {
        let postmove = board.move_piece(all_legal_moves.0[possible_move]);
        move_sum += perft_internal(postmove, depth + 1, max_depth, tables);
    }

    move_sum
}

pub fn perft(board: Board, depth: u8, tables: &ChessTables) -> usize {
    let mut sum = 0;
    let legal_moves = board.get_all_legal_moves(tables);

    for possible_move in 0..legal_moves.1 {
        let move_count = if depth == 1 {
            1
        } else {
            perft_internal(
                board.move_piece(legal_moves.0[possible_move]),
                1,
                depth - 1,
                tables,
            )
        };
        sum += move_count;
        let parsed = ChessMove::unpack(legal_moves.0[possible_move]);

        println!(
            "{}{} {}",
            human_readable_position(parsed.origin).to_lowercase(),
            human_readable_position(parsed.destination).to_lowercase(),
            move_count
        );
    }

    sum
}

fn generate_king_bitmask(
    tables: &ChessTables,
    friendly_occupancy: BitBoard,
    position: u8,
) -> BitBoard {
    let mut movement_mask =
        tables.lookup_tables[LookupTable::KingMoves as usize][position as usize];
    movement_mask &= !friendly_occupancy;
    movement_mask
}
