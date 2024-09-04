mod chess_data;

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

    lookup_tables: [[u64; 64]; 12], // This should be in some kind of meta object, not related directly to the rules/behavior of chess.
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
                new_board.clear_square(chess_move.destination);
                new_board.bitboards[piece_type as usize].set_bit(chess_move.destination);
            }
            MoveType::KingCastle => todo!(),
            MoveType::QueenCastle => todo!(),
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

    pub fn get_pseudolegal_capture_mask(&self, position: u8) -> u64 {
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

        let lookup = [0, 1, 2, 3, 4, 11, 0, 1, 2, 3, 4, 11];

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
                    & !occupancy;

                movement |= self.lookup_tables[LookupTable::WhitePawnCaptures as usize]
                    [position as usize]
                    & enemy_occupancy;
            }
            Pieces::BlackPawn => {
                movement |= self.lookup_tables[LookupTable::BlackPawnMoves as usize]
                    [position as usize]
                    & !occupancy;

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
        let psuedolegal_capture_mask = self.get_pseudolegal_capture_mask(position);
        let mut move_buffer = Vec::new();

        let piece = self.find_piece(position);
        if self.en_passant.is_some() {
            if piece == Pieces::WhitePawn
                && self.lookup_tables[LookupTable::WhitePawnCaptures as usize][position as usize]
                    & (1 << (self.en_passant.unwrap() - 8))
                    != 0
            {
                move_buffer.push(ChessMove::pack(&ChessMove {
                    position,
                    destination: self.en_passant.unwrap() - 8,
                    move_type: MoveType::EnPassant,
                }));
            }
            if piece == Pieces::BlackPawn
                && self.lookup_tables[LookupTable::BlackPawnCaptures as usize][position as usize]
                    & (1 << (self.en_passant.unwrap() + 8))
                    != 0
            {
                move_buffer.push(ChessMove::pack(&ChessMove {
                    position,
                    destination: self.en_passant.unwrap() + 8,
                    move_type: MoveType::EnPassant,
                }));
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
                    self.lookup_tables[LookupTable::WhitePawnLongMoves as usize][position as usize]
                        & (1 << destination)
                        != 0
                }
                Pieces::BlackPawn => {
                    self.lookup_tables[LookupTable::BlackPawnLongMoves as usize][position as usize]
                        & (1 << destination)
                        != 0
                }
                _ => false,
            };

            if is_long_move {
                move_buffer.push(ChessMove::pack(&ChessMove {
                    position,
                    destination: destination as u8,
                    move_type: MoveType::DoublePawnPush,
                }));
                continue;
            }

            if piece == Pieces::WhitePawn || piece == Pieces::BlackPawn {
                let last_rank = match piece {
                    Pieces::WhitePawn => 7,
                    Pieces::BlackPawn => 0,
                    _ => panic!(),
                };
                let is_moving_to_last_rank = (destination / 8) == last_rank;
                if is_moving_to_last_rank {
                    move_buffer.push(ChessMove::pack(&ChessMove {
                        position,
                        destination: destination as u8,
                        move_type: MoveType::QueenPromotion,
                    }));
                    move_buffer.push(ChessMove::pack(&ChessMove {
                        position,
                        destination: destination as u8,
                        move_type: MoveType::RookPromotion,
                    }));
                    move_buffer.push(ChessMove::pack(&ChessMove {
                        position,
                        destination: destination as u8,
                        move_type: MoveType::BishopPromotion,
                    }));
                    move_buffer.push(ChessMove::pack(&ChessMove {
                        position,
                        destination: destination as u8,
                        move_type: MoveType::KnightPromotion,
                    }));
                    continue;
                }
            }

            move_buffer.push(ChessMove::pack(&ChessMove {
                position,
                destination: destination as u8,
                move_type: MoveType::Capture,
            }));
        }

        move_buffer
    }

    pub fn try_make_move(&mut self, position: u8, destination: u8) {
        let legal_moves = self.get_legal_moves(position);
        for possible_move in legal_moves {
            let parsed_move = ChessMove::unpack(possible_move);
            dbg!(parsed_move);
            match parsed_move.move_type {
                MoveType::QueenPromotion => {}
                MoveType::RookPromotion => continue,
                MoveType::BishopPromotion => continue,
                MoveType::KnightPromotion => continue,

                _ => {}
            }
            if parsed_move.destination == destination {
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
            for enemy_piece_position in 0..64 {
                enemy_bitmask |= chess_move.get_pseudolegal_capture_mask(enemy_piece_position);
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

    #[cfg(test)]
    fn get_all_legal_moves(&self) -> Vec<u16> {
        let mut move_buffer = Vec::new();

        for position in 0..64 {
            move_buffer.extend(self.get_legal_moves(position));
        }

        move_buffer
    }
    pub fn fen_parser(fen: &str) -> Board {
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
            if split_fen[2].contains('k') {
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
fn perft_internal(board: Board, depth: u8, max_depth: u8) -> usize {
    let all_legal_moves = board.get_all_legal_moves();
    if depth == max_depth {
        return all_legal_moves.len();
    }
    if board.is_in_checkmate() {
        return all_legal_moves.len();
    }

    let mut move_sum = 0;

    for possible_move in &all_legal_moves {
        let postmove = board.move_piece(*possible_move);
        move_sum += perft_internal(postmove, depth + 1, max_depth);
    }

    move_sum
}

#[cfg(test)]
fn perft(board: Board, depth: u8) -> usize {
    let mut results = Vec::new();

    let mut sum = 0;
    let legal_moves = board.get_all_legal_moves();

    for possible_move in legal_moves {
        let move_count = if depth == 1 {
            1
        } else {
            perft_internal(board.move_piece(possible_move), 1, depth - 1)
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

    #[test]
    fn perft_base() {
        let board = Board::default();
        let move_count = perft(board, 5);
        assert_eq!(move_count, 4_865_609);
    }

    #[test]
    fn perft_no_castle() {
        let board = Board::fen_parser("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - ");
        let move_count = perft(board, 6);
        assert_eq!(move_count, 11_030_083);
    }

    #[test]
    fn perft_castling() {
        let board = Board::fen_parser(
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        );
        let move_count = perft(board, 4);
        assert_eq!(move_count, 4_185_552);
    }

    #[test]
    fn perft_promotion() {
        let board = Board::fen_parser("n1n5/PPPk4/8/8/8/8/4Kppp/5N1N b - - 0 1");
        let move_count = perft(board, 5);
        assert_eq!(move_count, 3_605_103);
    }
}
