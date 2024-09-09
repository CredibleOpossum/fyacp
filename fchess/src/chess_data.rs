#![allow(clippy::needless_range_loop)]
use std::collections::{HashMap, HashSet};

// The nice thing about bitboards is that it doesn't matter how you generate them as they are only calculated once, a lot of this is inefficient or strange
// This should probably be some form of bootstrapping instead of generating it on launch.
use crate::{data::*, ChessTables};
use crate::{BitBoard, RaycastTables, EMPTY};

pub fn generate_data() -> [[u64; 64]; 12] {
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
        [EMPTY; 64],
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

fn generate_long_pawn_moves(color: Color) -> [u64; 64] {
    let mut moves = [0; 64];

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
            *pawn_move = 1 << (position as i32 + direction * 2);
        }
    }

    moves
}

fn generate_pawn_moves(color: Color) -> [u64; 64] {
    // Includes invaild position, not sure how to handle.
    let mut moves = [0; 64];

    let direction: i32 = match color {
        Color::White => 8,
        Color::Black => -8,
    };

    for position in 0..64 {
        let mut movement = BitBoard(0);

        let position_bitmask = 1 << position;

        let on_edge = match color {
            Color::White => (position_bitmask & BOARD_TOP) != 0,
            Color::Black => (position_bitmask & BOARD_BOTTOM) != 0,
        };

        if !on_edge {
            movement.set_bit((position + direction).try_into().unwrap());
        }

        moves[position as usize] = movement.0;
    }
    moves
}

fn generate_pawn_captures(color: Color) -> [u64; 64] {
    // Pawns can't legally exist in certain locations which this code doesn't accoutn for, an illegal position shouldn't
    let mut moves = [0u64; 64];
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

        let position_bitmask = 1 << position;

        let is_on_top = position_bitmask & BOARD_TOP != 0;
        let is_on_bottom = position_bitmask & BOARD_BOTTOM != 0;
        let is_on_left_edge = position_bitmask & BOARD_LEFT != 0;
        let is_on_right_edge = position_bitmask & BOARD_RIGHT != 0;

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

        moves[position as usize] = movement.0;
    }

    moves
}

fn generate_king_moves() -> [u64; 64] {
    let mut moves = [0u64; 64];
    for position in 0..64 {
        let bit_position = 1 << position;

        let is_on_top = bit_position & BOARD_TOP != 0;
        let is_on_bottom = bit_position & BOARD_BOTTOM != 0;
        let is_on_left_edge = bit_position & BOARD_LEFT != 0;
        let is_on_right_edge = bit_position & BOARD_RIGHT != 0;

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

        moves[position as usize] = movement.0;
    }
    moves
}

const TOP: u64 = 0xff00000000000000;
const LEFT: u64 = 0x8080808080808080;
const RIGHT: u64 = 0x101010101010101;
const BOTTOM: u64 = 0xff;
fn generate_rook_moves_short() -> [u64; 64] {
    let tables = RaycastTables::new();
    let mut rook_moves = generate_rook_moves(&tables);

    for position in 0..64 {
        if (rook_moves[position] & TOP).count_ones() == 1 {
            rook_moves[position] &= !TOP;
        }
        if (rook_moves[position] & LEFT).count_ones() == 1 {
            rook_moves[position] &= !LEFT;
        }
        if (rook_moves[position] & RIGHT).count_ones() == 1 {
            rook_moves[position] &= !RIGHT;
        }
        if (rook_moves[position] & BOTTOM).count_ones() == 1 {
            rook_moves[position] &= !BOTTOM;
        }
    }

    rook_moves
}
fn generate_rook_moves(tables: &RaycastTables) -> [u64; 64] {
    let mut north = tables.north;
    let west = tables.west;
    let east = tables.east;
    let south = tables.south;

    for position in 0..64 {
        north[position] |= west[position] | east[position] | south[position];
    }

    north
}

fn generate_bishop_moves() -> [u64; 64] {
    let tables = RaycastTables::new();

    let mut north_west = tables.north_west;
    let north_east = tables.north_east;
    let south_west = tables.south_west;
    let south_east = tables.south_east;

    for position in 0..64 {
        north_west[position] |= north_east[position] | south_west[position] | south_east[position];
    }

    north_west
}

fn generate_bishop_moves_short() -> [u64; 64] {
    let mut rook_moves = generate_bishop_moves();

    for position in 0..64 {
        if (rook_moves[position] & TOP).count_ones() == 1 {
            rook_moves[position] &= !TOP;
        }
        if (rook_moves[position] & LEFT).count_ones() == 1 {
            rook_moves[position] &= !LEFT;
        }
        if (rook_moves[position] & RIGHT).count_ones() == 1 {
            rook_moves[position] &= !RIGHT;
        }
        if (rook_moves[position] & BOTTOM).count_ones() == 1 {
            rook_moves[position] &= !BOTTOM;
        }
    }

    rook_moves
}

fn generate_queen_moves() -> [u64; 64] {
    let tables = RaycastTables::new();
    let mut straight = generate_rook_moves(&tables);
    let diagonal = generate_bishop_moves();
    for index in 0..straight.len() {
        straight[index] |= diagonal[index];
    }

    straight
}

fn generate_knight_moves() -> [u64; 64] {
    let mut moves = [0u64; 64];
    for position in 0..64 {
        let mut movement = BitBoard(0);
        let bit_position = 1 << position;

        let is_on_top_thick = bit_position & THICK_BOARD_TOP != 0;
        let is_on_bottom_thick = bit_position & THICK_BOARD_BOTTOM != 0;
        let is_on_left_edge_thick = bit_position & THICK_BOARD_LEFT != 0;
        let is_on_right_edge_thick = bit_position & THICK_BOARD_RIGHT != 0;

        let is_on_top = bit_position & BOARD_TOP != 0;
        let is_on_bottom = bit_position & BOARD_BOTTOM != 0;
        let is_on_left_edge = bit_position & BOARD_LEFT != 0;
        let is_on_right_edge = bit_position & BOARD_RIGHT != 0;

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

        moves[position as usize] = movement.0;
    }
    moves
}
