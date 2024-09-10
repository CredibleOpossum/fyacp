mod chess_data;

use chess_data::generate_data;

mod data;
use data::*;

mod magics;

const MAX_LEGAL_MOVES: usize = 195; // A resonable limit.

#[derive(Debug)]
struct Moves([u16; MAX_LEGAL_MOVES]);
impl Default for Moves {
    fn default() -> Self {
        Moves([0_u16; MAX_LEGAL_MOVES])
    }
}

pub enum BoardState {
    Checkmate,
    Stalemate,
    OnGoing,
}

const EMPTY_STRING: String = String::new();
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
    lookup_tables: [[u64; 64]; 12],
    other_tables: RaycastTables,
}
impl Default for ChessTables {
    fn default() -> Self {
        ChessTables {
            lookup_tables: generate_data(),
            other_tables: RaycastTables::new(),
        }
    }
}

#[derive(Clone)]
pub struct Board {
    pub bitboards: [BitBoard; 12],
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

fn bishop_moves(position: u8, occupancy: u64, tables: &ChessTables) -> u64 {
    let movement_mask = tables.lookup_tables[LookupTable::BishopMoves as usize][position as usize]; // Short rook bitmask
    let key = ((movement_mask & occupancy).wrapping_mul(magics::MAGICS_BISHOP[position as usize]))
        >> magics::MAGIC_SHIFT_BISHOP;
    magics::LOOKUP_BISHOP[position as usize][key as usize]
}

fn rook_moves(position: u8, occupancy: u64, tables: &ChessTables) -> u64 {
    let movement_mask = tables.lookup_tables[LookupTable::RookMoves as usize][position as usize]; // Short rook bitmask
    let key = ((movement_mask & occupancy).wrapping_mul(magics::MAGICS_ROOK[position as usize]))
        >> magics::MAGIC_SHIFT_ROOK;
    magics::LOOKUP_ROOK[position as usize][key as usize]
}

impl Board {
    pub fn get_legal_movement_mask(&self, position: u8, tables: &ChessTables) -> u64 {
        let mut mask: u64 = 0;
        let moves = self.get_legal_moves(position, tables);
        for legal_move in 0..MAX_LEGAL_MOVES {
            if moves.0[legal_move] == 0 {
                break;
            }
            let legal_move_parsed = ChessMove::unpack(moves.0[legal_move]);
            mask |= 1 << legal_move_parsed.destination;
        }
        mask
    }

    /*
    fn print_board(&self) {
        let mut text_representation = self.get_text_representation();
        text_representation.reverse();

        println!("----------");
        for (char_pos, char) in text_representation.iter().enumerate() {
            if char_pos % 8 == 0 {
                println!();
            }
            if *char != String::default() {
                print!("{} ", char);
            } else {
                print!("  ");
            }
        }
        println!("\n----------");
    }
    */

    fn get_full_capture_mask(&self, color: Color, tables: &ChessTables) -> u64 {
        let mut board_capturemask = 0;
        for enemy_piece_position in 0..64 {
            board_capturemask |= self
                .get_pseudolegal_capture_mask(enemy_piece_position, color, tables)
                .0;
        }
        board_capturemask
    }

    fn is_in_check(&self, tables: &ChessTables) -> bool {
        let enemy_bitmask = self.get_full_capture_mask(self.other_color(), tables);

        (self.find_kind_bitboard(self.turn).0 & enemy_bitmask) != 0
    }

    pub fn get_board_state(&self, tables: &ChessTables) -> BoardState {
        for possible_move in 0..64 {
            if self.get_legal_moves(possible_move, tables).0[0] != 0 {
                return BoardState::OnGoing;
            }
        }

        if self.is_in_check(tables) {
            return BoardState::Checkmate;
        }

        BoardState::Stalemate
    }

    fn clear_square(&mut self, position: u8) {
        for bitboard in &mut self.bitboards {
            bitboard.clear_bit(position);
        }
    }

    fn find_piece(&self, position: u8) -> Pieces {
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

    pub fn other_color(&self) -> Color {
        match self.turn {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }

    fn move_piece(&self, chess_move: u16) -> Board {
        let chess_move = ChessMove::unpack(chess_move);

        let mut new_board = self.clone();
        let piece_type = new_board.find_piece(chess_move.position);

        new_board.bitboards[piece_type as usize].clear_bit(chess_move.position);

        match chess_move.move_type {
            MoveType::QuietMove => todo!(),
            MoveType::DoublePawnPush => {
                new_board.clear_square(chess_move.destination);
                new_board.bitboards[piece_type as usize].set_bit(chess_move.destination);
            }
            MoveType::KingCastle => {
                new_board.bitboards[piece_type as usize].set_bit(chess_move.destination);

                let rook_bitboard_location = match self.turn {
                    Color::White => Pieces::WhiteRook as usize,
                    Color::Black => Pieces::BlackRook as usize,
                };

                new_board.bitboards[rook_bitboard_location].clear_bit(chess_move.destination - 1);
                new_board.bitboards[rook_bitboard_location].set_bit(chess_move.destination + 1);
            }
            MoveType::QueenCastle => {
                new_board.bitboards[piece_type as usize].set_bit(chess_move.destination);

                let rook_bitboard_location = match self.turn {
                    Color::White => Pieces::WhiteRook as usize,
                    Color::Black => Pieces::BlackRook as usize,
                };

                new_board.bitboards[rook_bitboard_location].clear_bit(chess_move.destination + 2);
                new_board.bitboards[rook_bitboard_location].set_bit(chess_move.destination - 1);
            }
            MoveType::Capture => {
                new_board.clear_square(chess_move.destination);
                new_board.bitboards[piece_type as usize].set_bit(chess_move.destination);
            }
            MoveType::EnPassant => {
                let direction: i32 = match new_board.turn {
                    Color::White => -16,
                    Color::Black => 16,
                };
                new_board.clear_square((self.en_passant.unwrap() as i32 + direction) as u8);
                new_board.bitboards[piece_type as usize].set_bit(chess_move.destination);
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
            MoveType::RookPromotion => {
                new_board.clear_square(chess_move.destination);
                match self.turn {
                    Color::White => new_board.bitboards[Pieces::WhiteRook as usize]
                        .set_bit(chess_move.destination),
                    Color::Black => new_board.bitboards[Pieces::BlackRook as usize]
                        .set_bit(chess_move.destination),
                };
            }
            MoveType::BishopPromotion => {
                new_board.clear_square(chess_move.destination);
                match self.turn {
                    Color::White => new_board.bitboards[Pieces::WhiteBishop as usize]
                        .set_bit(chess_move.destination),
                    Color::Black => new_board.bitboards[Pieces::BlackBishop as usize]
                        .set_bit(chess_move.destination),
                };
            }
            MoveType::KnightPromotion => {
                new_board.clear_square(chess_move.destination);
                match self.turn {
                    Color::White => new_board.bitboards[Pieces::WhiteKnight as usize]
                        .set_bit(chess_move.destination),
                    Color::Black => new_board.bitboards[Pieces::BlackKnight as usize]
                        .set_bit(chess_move.destination),
                };
            }
        }

        if chess_move.move_type == MoveType::DoublePawnPush {
            new_board.en_passant = Some(chess_move.position);
        } else {
            new_board.en_passant = None;
        }

        match piece_type {
            Pieces::WhiteKing => {
                new_board.castling_rights.white_kingside = false;
                new_board.castling_rights.white_queenside = false;
            }

            Pieces::BlackKing => {
                new_board.castling_rights.black_kingside = false;
                new_board.castling_rights.black_queenside = false;
            }
            _ => {}
        }

        // Lets handle rook castling rights, if a rook is moved we need to get rid of castling for that rook.
        // It doesn't matter if a rook is being moved since the castling rights would already be gone.
        match chess_move.position {
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
    ) -> (u64, u64, u64) {
        let piece_type = self.find_piece(position);

        if piece_type == Pieces::None {
            return (0, 0, 0);
        }

        let piece_color = match piece_type as usize / 6 {
            0..1 => Color::White,
            1..2 => Color::Black,
            _ => panic!(),
        };

        if piece_color != color {
            return (0, 0, 0);
        }

        let mut friendly_occupancy = 0;
        let mut enemy_occupancy = 0;
        for index in 0..6 {
            friendly_occupancy |= self.bitboards[index].0;
        }
        for index in 6..12 {
            enemy_occupancy |= self.bitboards[index].0;
        }
        if color != Color::White {
            std::mem::swap(&mut friendly_occupancy, &mut enemy_occupancy);
        }
        let occupancy = friendly_occupancy | enemy_occupancy;

        let mut movement_mask = 0;
        match piece_type {
            Pieces::WhiteKing => {
                movement_mask =
                    tables.lookup_tables[LookupTable::KingMoves as usize][position as usize];
                movement_mask &= !friendly_occupancy;
            }
            Pieces::BlackKing => {
                movement_mask =
                    tables.lookup_tables[LookupTable::KingMoves as usize][position as usize];
                movement_mask &= !friendly_occupancy;
            }

            Pieces::WhitePawn => {
                movement_mask |= tables.lookup_tables[LookupTable::WhitePawnMoves as usize]
                    [position as usize]
                    & !occupancy;

                if tables.lookup_tables[LookupTable::WhitePawnMoves as usize][position as usize]
                    & occupancy
                    == 0
                {
                    movement_mask |= tables.lookup_tables[LookupTable::WhitePawnLongMoves as usize]
                        [position as usize]
                        & !occupancy;
                }
                movement_mask |=
                    tables.lookup_tables[LookupTable::WhitePawnCaptures as usize][position as usize]
            }

            Pieces::BlackPawn => {
                movement_mask |= tables.lookup_tables[LookupTable::BlackPawnMoves as usize]
                    [position as usize]
                    & !occupancy;

                if tables.lookup_tables[LookupTable::BlackPawnMoves as usize][position as usize]
                    & occupancy
                    == 0
                {
                    movement_mask |= tables.lookup_tables[LookupTable::BlackPawnLongMoves as usize]
                        [position as usize]
                        & !occupancy;
                }
                movement_mask |= tables.lookup_tables[LookupTable::BlackPawnCaptures as usize]
                    [position as usize]
                    & enemy_occupancy;
            }

            Pieces::WhiteKnight => {
                movement_mask =
                    tables.lookup_tables[LookupTable::KnightMoves as usize][position as usize];
                movement_mask &= !friendly_occupancy;
            }
            Pieces::BlackKnight => {
                movement_mask =
                    tables.lookup_tables[LookupTable::KnightMoves as usize][position as usize];
                movement_mask &= !friendly_occupancy;
            }

            Pieces::WhiteRook | Pieces::BlackRook => {
                movement_mask = rook_moves(position, occupancy, tables);
                movement_mask &= !friendly_occupancy;
            }
            Pieces::WhiteBishop | Pieces::BlackBishop => {
                movement_mask = bishop_moves(position, occupancy, tables);
                movement_mask &= !friendly_occupancy;
            }
            Pieces::WhiteQueen | Pieces::BlackQueen => {
                movement_mask = rook_moves(position, occupancy, tables)
                    | bishop_moves(position, occupancy, tables);
                movement_mask &= !friendly_occupancy;
            }
            Pieces::None => panic!(),
        }
        (movement_mask, friendly_occupancy, enemy_occupancy)
    }

    fn get_pseudolegal_moves(&self, position: u8, tables: &ChessTables) -> Moves {
        let (psuedolegal_capture_mask, friendly_occupacny, enemy_occupancy) =
            self.get_pseudolegal_capture_mask(position, self.turn, tables);
        let mut move_buffer = Moves::default();
        let mut move_position = 0;

        let piece = self.find_piece(position);
        if self.en_passant.is_some() {
            if piece == Pieces::WhitePawn
                && tables.lookup_tables[LookupTable::WhitePawnCaptures as usize][position as usize]
                    & (1 << (self.en_passant.unwrap() - 8))
                    != 0
            {
                move_buffer.0[move_position] = ChessMove::pack(&ChessMove {
                    position,
                    destination: self.en_passant.unwrap() - 8,
                    move_type: MoveType::EnPassant,
                });
                move_position += 1;
            }
            if piece == Pieces::BlackPawn
                && tables.lookup_tables[LookupTable::BlackPawnCaptures as usize][position as usize]
                    & (1 << (self.en_passant.unwrap() + 8))
                    != 0
            {
                move_buffer.0[move_position] = ChessMove::pack(&ChessMove {
                    position,
                    destination: self.en_passant.unwrap() + 8,
                    move_type: MoveType::EnPassant,
                });
                move_position += 1;
            }
        }

        for destination in 0..64 {
            let destination_mask = 1 << destination;
            if psuedolegal_capture_mask & destination_mask == 0 {
                continue; // Not psuedolegal
            }
            let piece = self.find_piece(position);

            let is_long_move = match piece {
                Pieces::WhitePawn => {
                    tables.lookup_tables[LookupTable::WhitePawnLongMoves as usize]
                        [position as usize]
                        & (1 << destination)
                        != 0
                }
                Pieces::BlackPawn => {
                    tables.lookup_tables[LookupTable::BlackPawnLongMoves as usize]
                        [position as usize]
                        & (1 << destination)
                        != 0
                }
                _ => false,
            };

            if is_long_move {
                move_buffer.0[move_position] = ChessMove::pack(&ChessMove {
                    position,
                    destination,
                    move_type: MoveType::DoublePawnPush,
                });
                move_position += 1;
                continue;
            }

            if piece == Pieces::WhitePawn || piece == Pieces::BlackPawn {
                let is_pawn_capture = (position % 8) != (destination % 8);
                if is_pawn_capture && (((1 << destination) & enemy_occupancy) == 0) {
                    continue; // This isn't a legal pawn move, since it's capturing but not hitting an enemy.
                }
                let last_rank = match piece {
                    Pieces::WhitePawn => 7,
                    Pieces::BlackPawn => 0,
                    _ => panic!(),
                };
                let is_moving_to_last_rank = (destination / 8) == last_rank;
                if is_moving_to_last_rank {
                    move_buffer.0[move_position] = ChessMove::pack(&ChessMove {
                        position,
                        destination,
                        move_type: MoveType::QueenPromotion,
                    });
                    move_position += 1;
                    move_buffer.0[move_position] = ChessMove::pack(&ChessMove {
                        position,
                        destination,
                        move_type: MoveType::RookPromotion,
                    });
                    move_position += 1;
                    move_buffer.0[move_position] = ChessMove::pack(&ChessMove {
                        position,
                        destination,
                        move_type: MoveType::BishopPromotion,
                    });
                    move_position += 1;
                    move_buffer.0[move_position] = ChessMove::pack(&ChessMove {
                        position,
                        destination,
                        move_type: MoveType::KnightPromotion,
                    });
                    move_position += 1;
                    continue;
                }
            }

            move_buffer.0[move_position] = ChessMove::pack(&ChessMove {
                position,
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
        let white_kingside_hitmask: u64 = 0xe;
        let white_queenside_hitmask: u64 = 0x38;
        let black_kingside_hitmask: u64 = 0xe00000000000000;
        let black_queenside_hitmask: u64 = 0x3800000000000000;

        let white_kingside_hitmask_friendly: u64 = 0x6;
        let white_queenside_hitmask_friendly: u64 = 0x70;
        let black_kingside_hitmask_friendly: u64 = 0x600000000000000;
        let black_queenside_hitmask_friendly: u64 = 0x7000000000000000;

        match piece {
            Pieces::WhiteKing => {
                if self.turn == Color::White {
                    if self.castling_rights.white_kingside
                        && (enemy_hitmask & white_kingside_hitmask) == 0
                        && (blocking_pieces & white_kingside_hitmask_friendly) == 0
                    {
                        move_buffer.0[move_position] = ChessMove::pack(&ChessMove {
                            position: white_king_location_x,
                            destination: white_king_location_x - 2,
                            move_type: MoveType::KingCastle,
                        });
                        move_position += 1;
                    }
                    if self.castling_rights.white_queenside
                        && (enemy_hitmask & white_queenside_hitmask) == 0
                        && (blocking_pieces & white_queenside_hitmask_friendly) == 0
                    {
                        move_buffer.0[move_position] = ChessMove::pack(&ChessMove {
                            position: white_king_location_x,
                            destination: white_king_location_x + 2,
                            move_type: MoveType::QueenCastle,
                        });
                    }
                }
            }
            Pieces::BlackKing => {
                if self.turn == Color::Black {
                    if self.castling_rights.black_kingside
                        && (enemy_hitmask & black_kingside_hitmask) == 0
                        && (blocking_pieces & black_kingside_hitmask_friendly) == 0
                    {
                        move_buffer.0[move_position] = ChessMove::pack(&ChessMove {
                            position: black_king_location_x,
                            destination: black_king_location_x - 2,
                            move_type: MoveType::KingCastle,
                        });
                        move_position += 1;
                    }

                    if self.castling_rights.black_queenside
                        && (enemy_hitmask & black_queenside_hitmask) == 0
                        && (blocking_pieces & black_queenside_hitmask_friendly) == 0
                    {
                        move_buffer.0[move_position] = ChessMove::pack(&ChessMove {
                            position: black_king_location_x,
                            destination: black_king_location_x + 2,
                            move_type: MoveType::QueenCastle,
                        });
                    }
                }
            }
            _ => {}
        }

        move_buffer
    }

    pub fn try_make_move(&mut self, position: u8, destination: u8, tables: &ChessTables) {
        let legal_moves = self.get_legal_moves(position, tables);
        for possible_move in 0..MAX_LEGAL_MOVES {
            if legal_moves.0[possible_move] == 0 {
                break;
            }
            let parsed_move = ChessMove::unpack(legal_moves.0[possible_move]);
            match parsed_move.move_type {
                MoveType::QueenPromotion => {}
                MoveType::RookPromotion => continue,
                MoveType::BishopPromotion => continue,
                MoveType::KnightPromotion => continue,

                _ => {}
            }
            if parsed_move.destination == destination {
                *self = self.move_piece(legal_moves.0[possible_move]);
            }
        }
    }

    fn find_kind_bitboard(&self, color: Color) -> BitBoard {
        match color {
            Color::White => self.bitboards[0],
            Color::Black => self.bitboards[6],
        }
    }

    fn get_legal_moves(&self, position: u8, tables: &ChessTables) -> Moves {
        let psuedo_legal_moves = self.get_pseudolegal_moves(position, tables);
        //dbg!(psuedo_legal_moves.0[0]);
        let mut legal_move_buffer = Moves::default();
        let mut legal_move_index = 0;

        for psuedo_legal_move_index in 0..MAX_LEGAL_MOVES {
            if psuedo_legal_moves.0[psuedo_legal_move_index] == 0 {
                // We hit a empty move, no other moves should be in front of it.
                break;
            }
            let chess_move = self.move_piece(psuedo_legal_moves.0[psuedo_legal_move_index]);

            let king_bitmask = chess_move.find_kind_bitboard(chess_move.other_color());

            let mut enemy_bitmask = 0;
            for enemy_piece_position in 0..64 {
                enemy_bitmask |= chess_move
                    .get_pseudolegal_capture_mask(enemy_piece_position, chess_move.turn, tables)
                    .0;
            }

            if enemy_bitmask & king_bitmask.0 == 0 {
                legal_move_buffer.0[legal_move_index] =
                    psuedo_legal_moves.0[psuedo_legal_move_index];
                legal_move_index += 1;
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

    #[cfg(test)]
    fn get_all_legal_moves(&self, tables: &ChessTables) -> Vec<u16> {
        let mut move_buffer: Vec<u16> = Vec::new();

        for position in 0..64 {
            let moves = self.get_legal_moves(position, tables);
            for move_index in 0..MAX_LEGAL_MOVES {
                if moves.0[move_index] == 0 {
                    break;
                }
                move_buffer.push(moves.0[move_index]);
            }
        }

        move_buffer
    }
    pub fn fen_parser(fen: &str) -> Board {
        // Doesn't get parse en_passant square.
        let mut board = Board::default();

        let mut index: usize = 0;
        let split_fen: Vec<&str> = fen.split(' ').collect();

        for bitboard in 0..board.bitboards.len() {
            board.bitboards[bitboard] = BitBoard(0);
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
                'P' => board.bitboards[Pieces::WhitePawn as usize].0 |= 1 << index,
                'p' => board.bitboards[Pieces::BlackPawn as usize].0 |= 1 << index,

                'N' => board.bitboards[Pieces::WhiteKnight as usize].0 |= 1 << index,
                'n' => board.bitboards[Pieces::BlackKnight as usize].0 |= 1 << index,

                'B' => board.bitboards[Pieces::WhiteBishop as usize].0 |= 1 << index,
                'b' => board.bitboards[Pieces::BlackBishop as usize].0 |= 1 << index,

                'R' => board.bitboards[Pieces::WhiteRook as usize].0 |= 1 << index,
                'r' => board.bitboards[Pieces::BlackRook as usize].0 |= 1 << index,

                'Q' => board.bitboards[Pieces::WhiteQueen as usize].0 |= 1 << index,
                'q' => board.bitboards[Pieces::BlackQueen as usize].0 |= 1 << index,

                'K' => board.bitboards[Pieces::WhiteKing as usize].0 |= 1 << index,
                'k' => board.bitboards[Pieces::BlackKing as usize].0 |= 1 << index,

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

#[cfg(test)]
fn perft_internal(board: Board, depth: u8, max_depth: u8, tables: &ChessTables) -> usize {
    let all_legal_moves = board.get_all_legal_moves(tables);
    if depth == max_depth {
        return all_legal_moves.len();
    }

    match board.get_board_state(tables) {
        BoardState::Checkmate => return all_legal_moves.len(),
        BoardState::Stalemate => return all_legal_moves.len(),
        BoardState::OnGoing => {}
    }

    let mut move_sum = 0;

    for possible_move in &all_legal_moves {
        let postmove = board.move_piece(*possible_move);
        move_sum += perft_internal(postmove, depth + 1, max_depth, tables);
    }

    move_sum
}

#[cfg(test)]
fn perft(board: Board, depth: u8, tables: &ChessTables) -> usize {
    let mut results = Vec::new();

    let mut sum = 0;
    let legal_moves = board.get_all_legal_moves(tables);

    for possible_move in legal_moves {
        let move_count = if depth == 1 {
            1
        } else {
            perft_internal(board.move_piece(possible_move), 1, depth - 1, tables)
        };
        sum += move_count;
        let parsed = ChessMove::unpack(possible_move);

        results.push(format!(
            "{}{}: {}",
            human_readable_position(parsed.position),
            human_readable_position(parsed.destination),
            move_count
        ))
    }

    results.sort(); // Maybe just do it in order so I can live print the results similar to stockfish's CLI.
    for result in results {
        println!("{}", result);
    }

    sum
}

pub fn human_readable_position(position: u8) -> String {
    let first = match position % 8 {
        0 => 'h',
        1 => 'g',
        2 => 'f',
        3 => 'e',
        4 => 'd',
        5 => 'c',
        6 => 'b',
        7 => 'a',

        _ => panic!(),
    };

    let second = match position / 8 {
        0 => '1',
        1 => '2',
        2 => '3',
        3 => '4',
        4 => '5',
        5 => '6',
        6 => '7',
        7 => '8',

        _ => panic!(),
    };

    format!("{}{}", first, second)
}

#[cfg(test)]
mod tests {
    use super::*;
    // https://www.chessprogramming.org/Perft_Results

    /*
    #[test]
    fn perft_castling() {
        let tables = ChessTables::default();
        let board =
            Board::fen_parser("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - ");
        let move_count = perft(board, 5, &tables);
        assert_eq!(move_count, 193_690_690);
    }
    */

    #[test]
    fn perft_castling() {
        let tables = ChessTables::default();
        let board =
            Board::fen_parser("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - ");
        let move_count = perft(board, 4, &tables);
        assert_eq!(move_count, 4_085_603);
    }

    #[test]
    fn perft_base() {
        let tables = ChessTables::default();
        let board = Board::default();
        let move_count = perft(board, 5, &tables);
        assert_eq!(move_count, 4_865_609);
    }

    #[test]
    fn perft_no_castle() {
        let tables = ChessTables::default();
        let board = Board::fen_parser("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - ");
        let move_count = perft(board, 6, &tables);
        assert_eq!(move_count, 11_030_083);
    }

    #[test]
    fn perft_strange() {
        let tables = ChessTables::default();
        let board =
            Board::fen_parser("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1");
        let move_count = perft(board, 5, &tables);
        assert_eq!(move_count, 15_833_292);
    }

    #[test]
    fn perft_promotion() {
        let tables = ChessTables::default();
        let board = Board::fen_parser("n1n5/PPPk4/8/8/8/8/4Kppp/5N1N b - - 0 1");
        let move_count = perft(board, 5, &tables);
        assert_eq!(move_count, 3_605_103);
    }

    #[test]
    fn perft_promotion_pinned() {
        let tables = ChessTables::default();
        let board =
            Board::fen_parser("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8  ");
        let move_count = perft(board, 4, &tables);
        assert_eq!(move_count, 2_103_487);
    }
}
