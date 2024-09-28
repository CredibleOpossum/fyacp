#![allow(clippy::needless_range_loop)]
// The nice thing about bitboards is that it doesn't matter how you generate them as they are only calculated once, a lot of this is inefficient or strange
// This should probably be some form of bootstrapping instead of generating it on launch.
use crate::{bitboard::BitBoard, constants::*, structs::*};
use crate::{Board, RaycastTables};

pub fn generate_data() -> [[BitBoard; 64]; 12] {
    /*
    An array is basically required for performance / ergonomics but I should get rid of magic indexes
    Currently, the lookup table and the actual table could be desynced and cause painful bugs.
    */
    [
        generate_king_moves(),
        generate_queen_moves(),
        generate_rook_moves_short(),   // Magic friendly
        generate_bishop_moves_short(), // Magic friendly
        generate_knight_moves(),
        generate_pawn_moves(Color::White),
        generate_pawn_captures(Color::White),
        generate_pawn_moves(Color::Black),
        generate_pawn_captures(Color::Black),
        generate_long_pawn_moves(Color::White),
        generate_long_pawn_moves(Color::Black),
        [BitBoard(0); 64],
    ]
}

#[allow(dead_code)]
pub enum LookupTable {
    KingMoves,
    QueenMoves,
    RookMoves,
    BishopMoves,
    KnightMoves,
    WhitePawnMoves,
    WhitePawnCaptures,
    BlackPawnMoves,
    BlackPawnCaptures,
    WhitePawnLongMoves,
    BlackPawnLongMoves,
    Blank,
}

fn generate_long_pawn_moves(color: Color) -> [BitBoard; 64] {
    let mut moves = [BitBoard(0); 64];

    for (position, pawn_move) in moves.iter_mut().enumerate() {
        let direction: i32 = match color {
            Color::White => 8,
            Color::Black => -8,
        };
        let on_rank = match color {
            Color::White => position / 8 == 1,
            Color::Black => position / 8 == 6,
        };
        if on_rank {
            *pawn_move = BitBoard(1 << (position as i32 + direction * 2));
        }
    }

    moves
}

fn generate_pawn_moves(color: Color) -> [BitBoard; 64] {
    // Includes invaild position, not sure how to handle.
    let mut moves = [BitBoard(0); 64];

    let direction: i32 = match color {
        Color::White => 8,
        Color::Black => -8,
    };

    for position in 0..64 {
        let mut movement = BitBoard(0);

        let position_bitmask = BitBoard(1 << position);

        let on_edge = match color {
            Color::White => !(position_bitmask & BOARD_TOP).is_empty(),
            Color::Black => !(position_bitmask & BOARD_BOTTOM).is_empty(),
        };

        if !on_edge {
            movement.set_bit((position + direction).try_into().unwrap());
        }

        moves[position as usize] = movement;
    }
    moves
}

fn generate_pawn_captures(color: Color) -> [BitBoard; 64] {
    // Pawns can't legally exist in certain locations which this code doesn't accoutn for, an illegal position shouldn't
    let mut moves = [BitBoard(0); 64];
    for position in 0..64 {
        let mut movement = BitBoard(0);

        let direction_first: i32 = match color {
            Color::White => 9,
            Color::Black => -7,
        };
        let direction_second: i32 = match color {
            Color::White => 7,
            Color::Black => -9,
        };

        let position_bitmask = BitBoard(1 << position);

        let is_on_top = !(position_bitmask & BOARD_TOP).is_empty();
        let is_on_bottom = !(position_bitmask & BOARD_BOTTOM).is_empty();
        let is_on_left_edge = !(position_bitmask & BOARD_LEFT).is_empty();
        let is_on_right_edge = !(position_bitmask & BOARD_RIGHT).is_empty();

        let is_vaild = match color {
            Color::White => !is_on_top,
            Color::Black => !is_on_bottom,
        };

        if is_vaild && !is_on_left_edge {
            movement.set_bit((position + direction_first).try_into().unwrap());
        }

        if is_vaild && !is_on_right_edge {
            movement.set_bit((position + direction_second).try_into().unwrap());
        }

        moves[position as usize] = movement;
    }

    moves
}

fn generate_king_moves() -> [BitBoard; 64] {
    let mut moves = [BitBoard(0); 64];
    for position in 0..64 {
        let bit_position = BitBoard(1 << position);

        let is_on_top = !(bit_position & BOARD_TOP).is_empty();
        let is_on_bottom = !(bit_position & BOARD_BOTTOM).is_empty();
        let is_on_left_edge = !(bit_position & BOARD_LEFT).is_empty();
        let is_on_right_edge = !(bit_position & BOARD_RIGHT).is_empty();

        let mut movement = BitBoard(0);
        if !is_on_top && !is_on_left_edge {
            movement.set_bit(position + 9)
        }
        if !is_on_top {
            movement.set_bit(position + 8)
        }
        if !is_on_top && !is_on_right_edge {
            movement.set_bit(position + 7)
        }

        if !is_on_left_edge {
            movement.set_bit(position + 1)
        }

        if !is_on_right_edge {
            movement.set_bit(position - 1)
        }

        if !is_on_bottom && !is_on_left_edge {
            movement.set_bit(position - 7)
        }
        if !is_on_bottom {
            movement.set_bit(position - 8)
        }
        if !is_on_bottom && !is_on_right_edge {
            movement.set_bit(position - 9)
        }

        moves[position as usize] = movement;
    }
    moves
}

fn generate_rook_moves_short() -> [BitBoard; 64] {
    let tables = RaycastTables::default();
    let mut rook_moves = generate_rook_moves(&tables);

    for position in 0..64 {
        if (rook_moves[position] & TOP).0.count_ones() == 1 {
            rook_moves[position] &= !TOP;
        }
        if (rook_moves[position] & LEFT).0.count_ones() == 1 {
            rook_moves[position] &= !LEFT;
        }
        if (rook_moves[position] & RIGHT).0.count_ones() == 1 {
            rook_moves[position] &= !RIGHT;
        }
        if (rook_moves[position] & BOTTOM).0.count_ones() == 1 {
            rook_moves[position] &= !BOTTOM;
        }
    }

    rook_moves
}
fn generate_rook_moves(tables: &RaycastTables) -> [BitBoard; 64] {
    let mut rook_tables = [BitBoard(0); 64];

    let north = tables.north;
    let west = tables.west;
    let east = tables.east;
    let south = tables.south;

    for position in 0..64 {
        rook_tables[position] =
            BitBoard(north[position] | west[position] | east[position] | south[position]);
    }

    rook_tables
}

fn generate_bishop_moves() -> [BitBoard; 64] {
    let tables = RaycastTables::default();

    let mut bishop_moves = [BitBoard(0); 64];

    let north_west = tables.north_west;
    let north_east = tables.north_east;
    let south_west = tables.south_west;
    let south_east = tables.south_east;

    for position in 0..64 {
        bishop_moves[position] = BitBoard(
            north_west[position]
                | north_east[position]
                | south_west[position]
                | south_east[position],
        );
    }

    bishop_moves
}

fn generate_bishop_moves_short() -> [BitBoard; 64] {
    let mut rook_moves = generate_bishop_moves();

    for position in 0..64 {
        if (rook_moves[position] & TOP).0.count_ones() == 1 {
            rook_moves[position] &= !TOP;
        }
        if (rook_moves[position] & LEFT).0.count_ones() == 1 {
            rook_moves[position] &= !LEFT;
        }
        if (rook_moves[position] & RIGHT).0.count_ones() == 1 {
            rook_moves[position] &= !RIGHT;
        }
        if (rook_moves[position] & BOTTOM).0.count_ones() == 1 {
            rook_moves[position] &= !BOTTOM;
        }
    }

    rook_moves
}

fn generate_queen_moves() -> [BitBoard; 64] {
    let tables = RaycastTables::default();
    let mut straight = generate_rook_moves(&tables);
    let diagonal = generate_bishop_moves();
    for index in 0..straight.len() {
        straight[index] |= diagonal[index];
    }

    straight
}

fn generate_knight_moves() -> [BitBoard; 64] {
    let mut moves = [BitBoard(0); 64];
    for position in 0..64 {
        let mut movement = BitBoard(0);
        let bit_position = BitBoard(1 << position);

        let is_on_top_thick = !(bit_position & THICK_BOARD_TOP).is_empty();
        let is_on_bottom_thick = !(bit_position & THICK_BOARD_BOTTOM).is_empty();
        let is_on_left_edge_thick = !(bit_position & THICK_BOARD_LEFT).is_empty();
        let is_on_right_edge_thick = !(bit_position & THICK_BOARD_RIGHT).is_empty();

        let is_on_top = !(bit_position & BOARD_TOP).is_empty();
        let is_on_bottom = !(bit_position & BOARD_BOTTOM).is_empty();
        let is_on_left_edge = !(bit_position & BOARD_LEFT).is_empty();
        let is_on_right_edge = !(bit_position & BOARD_RIGHT).is_empty();

        if !is_on_top_thick && !is_on_left_edge {
            movement.set_bit(position + 17);
        }

        if !is_on_top_thick && !is_on_right_edge {
            movement.set_bit(position + 15);
        }

        if !is_on_top && !is_on_left_edge_thick {
            movement.set_bit(position + 10);
        }

        if !is_on_top && !is_on_right_edge_thick {
            movement.set_bit(position + 6);
        }

        if !is_on_bottom_thick && !is_on_right_edge {
            movement.set_bit(position - 17);
        }

        if !is_on_bottom_thick && !is_on_left_edge {
            movement.set_bit(position - 15);
        }

        if !is_on_bottom && !is_on_right_edge_thick {
            movement.set_bit(position - 10);
        }

        if !is_on_bottom && !is_on_left_edge_thick {
            movement.set_bit(position - 6);
        }

        moves[position as usize] = movement;
    }
    moves
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

    let en_passant_square_string = split_fen[3];
    if en_passant_square_string != "-" {
        let board_square_index = HUMAN_READBLE_SQAURES
            .iter()
            .position(|&r| r == en_passant_square_string.to_uppercase())
            .expect("En passant square wasn't vaild.") as u8;
        board.en_passant = Some(board_square_index)
    }

    let turn = match split_fen[1] {
        "w" => Color::White,
        "b" => Color::Black,
        _ => panic!("Invaild fen, incorrect turn?"),
    };

    board.turn = turn;

    board
}
