use std::default;

use colored::Colorize;

pub static EMPTY: u64 = 0;
pub static UNIVERSE: u64 = u64::MAX;

static BOARD_SIZE: usize = 64;

pub static BOARD_TOP: u64 = 0xFF00000000000000;
pub static BOARD_BOTTOM: u64 = 0xFF;
pub static BOARD_LEFT: u64 = 0x8080808080808080;
pub static BOARD_RIGHT: u64 = 0x101010101010101;

// For knight moves
pub static THICK_BOARD_TOP: u64 = 0xFFFF000000000000;
pub static THICK_BOARD_BOTTOM: u64 = 0xFFFF;
pub static THICK_BOARD_LEFT: u64 = 0xC0C0C0C0C0C0C0C0;
pub static THICK_BOARD_RIGHT: u64 = 0x303030303030303;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Color {
    White,
    Black,
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum MoveType {
    QuietMove,
    DoublePawnPush,
    KingCastle,
    QueenCastle,
    Capture,
    EnPassant,
    QueenPromotion,
    RookPromotion,
    BishopPromotion,
    Knight,
}

#[derive(Clone, Copy, Debug)]
pub struct ChessMove {
    pub position: u8,
    pub destination: u8,
    pub move_type: MoveType,
}
impl ChessMove {
    pub fn pack(&self) -> u16 {
        let position: u16 = (self.position as u16 & 0b111111) << 10; // 6 bits
        let destination: u16 = (self.destination as u16 & 0b111111) << 4; // 6 bits
        let move_type: u16 = self.move_type as u16 & 0b1111; // 4 bits

        position | destination | move_type
    }

    pub fn unpack(packed_move: u16) -> ChessMove {
        let position_bitmask: u16 = 0b1111110000000000;
        let destination_bitmask: u16 = 0b0000001111110000;
        let move_type_bitmask: u16 = 0b0000000000001111;

        let position: u8 = ((packed_move & position_bitmask) >> 10) as u8;
        let destination: u8 = ((packed_move & destination_bitmask) >> 4) as u8;
        let move_type: u8 = (packed_move & move_type_bitmask) as u8;

        let move_type = match move_type {
            0 => MoveType::QuietMove,
            1 => MoveType::DoublePawnPush,
            2 => MoveType::KingCastle,
            3 => MoveType::QueenCastle,
            4 => MoveType::Capture,
            5 => MoveType::EnPassant,
            6 => MoveType::QueenPromotion,
            7 => MoveType::RookPromotion,
            8 => MoveType::BishopPromotion,
            9 => MoveType::Knight,
            _ => panic!(),
        };

        ChessMove {
            position,
            destination,
            move_type,
        }
    }
}

pub static STARTING_POSITION: [BitBoard; 12] = [
    BitBoard(8),
    BitBoard(16),
    BitBoard(129),
    BitBoard(36),
    BitBoard(66),
    BitBoard(65280),
    BitBoard(576460752303423488),
    BitBoard(1152921504606846976),
    BitBoard(9295429630892703744),
    BitBoard(2594073385365405696),
    BitBoard(4755801206503243776),
    BitBoard(71776119061217280),
];

#[derive(Clone, Copy, PartialEq)]
pub enum Pieces {
    WhiteKing,
    WhiteQueen,
    WhiteRook,
    WhiteBishop,
    WhiteKnight,
    WhitePawn,

    BlackKing,
    BlackQueen,
    BlackRook,
    BlackBishop,
    BlackKnight,
    BlackPawn,

    None,
}
impl Pieces {
    pub fn from_u8(index: u8) -> Pieces {
        match index {
            0 => Pieces::WhiteKing,
            1 => Pieces::WhiteQueen,
            2 => Pieces::WhiteRook,
            3 => Pieces::WhiteBishop,
            4 => Pieces::WhiteKnight,
            5 => Pieces::WhitePawn,

            6 => Pieces::BlackKing,
            7 => Pieces::BlackQueen,
            8 => Pieces::BlackRook,
            9 => Pieces::BlackBishop,
            10 => Pieces::BlackKnight,
            11 => Pieces::BlackPawn,

            12 => Pieces::None,

            _ => panic!("Attmpted to obtain piece from invaild id."),
        }
    }
}

fn vaild_position(position: [i32; 2]) -> bool {
    (0..8).contains(&position[0]) && (0..8).contains(&position[1])
}

fn position_flatten(position: [i32; 2]) -> u8 {
    ((position[0] % 8) + position[1] * 8) as u8
}

fn calculate_sliding(direction: [i32; 2]) -> [u64; 64] {
    let mut table = [0; 64];
    for position in 0..BOARD_SIZE {
        let mut bitmap = BitBoard(0);

        let mut current_position = [position as i32 % 8, position as i32 / 8];

        loop {
            current_position[0] += direction[0];
            current_position[1] += direction[1];

            let is_vaild_position = vaild_position(current_position);

            if is_vaild_position {
                bitmap.set_bit(position_flatten(current_position));
            } else {
                break;
            }
        }

        table[position] = bitmap.0;
    }
    table
}

#[derive(Clone, Copy)]
pub struct RaycastTables {
    pub north_west: [u64; 64],
    pub north: [u64; 64],
    pub north_east: [u64; 64],

    pub west: [u64; 64],
    pub east: [u64; 64],

    pub south_west: [u64; 64],
    pub south: [u64; 64],
    pub south_east: [u64; 64],
}

impl RaycastTables {
    pub fn new() -> RaycastTables {
        RaycastTables {
            north_west: calculate_sliding([1, 1]),
            north: calculate_sliding([0, 1]),
            north_east: calculate_sliding([-1, 1]),

            west: calculate_sliding([1, 0]),
            east: calculate_sliding([-1, 0]),

            south_west: calculate_sliding([1, -1]),
            south: calculate_sliding([0, -1]),
            south_east: calculate_sliding([-1, -1]),
        }
    }
}

#[derive(Clone, Copy)]
pub struct BitBoard(pub u64);

impl BitBoard {
    pub fn set_bit(&mut self, bit_index: u8) {
        self.0 |= 1 << bit_index;
    }
    pub fn clear_bit(&mut self, bit_index: u8) {
        self.0 &= UNIVERSE ^ (1 << bit_index);
    }
    pub fn get_bit(&self, bit_index: u8) -> bool {
        self.0 & (1 << bit_index) != 0
    }

    fn print_internal(&self, highlighted_position: Option<u8>) {
        for bit in (0..64).rev() {
            // This is horrifying, probably should rework.
            let should_be_highlighted = if let Some(position) = highlighted_position {
                position == bit
            } else {
                false
            };

            let bit_value = self.get_bit(bit);
            if bit_value {
                print!("{} ", (bit_value as i32).to_string().green());
            } else if should_be_highlighted {
                print!("{} ", (bit_value as i32).to_string().yellow());
            } else {
                print!("{} ", (bit_value as i32).to_string().red());
            }

            if bit % 8 == 0 {
                println!();
            }
        }
        println!("{}", self.0);
    }

    pub fn print(&self) {
        self.print_internal(None);
    }
    pub fn print_highlighting(&self, position: u8) {
        self.print_internal(Some(position));
    }
}

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
