use crate::{bitboard::BitBoard, constants::*};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Color {
    White,
    Black,
}

impl Color {
    pub fn opposite(&self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}
//
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
    KnightPromotion,
}

#[derive(Clone, Copy, Debug)]
pub struct ChessMove {
    pub origin: u8,
    pub destination: u8,
    pub move_type: MoveType,
}
impl ChessMove {
    pub fn pack(&self) -> u16 {
        let position: u16 = (self.origin as u16 & 0b111111) << 10; // 6 bits
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
            9 => MoveType::KnightPromotion,
            _ => panic!(),
        };

        ChessMove {
            origin: position,
            destination,
            move_type,
        }
    }
}

pub static STARTING_POSITION: [[BitBoard; 6]; 2] = [
    [
        BitBoard(8),
        BitBoard(16),
        BitBoard(129),
        BitBoard(36),
        BitBoard(66),
        BitBoard(65280),
    ],
    [
        BitBoard(576460752303423488),
        BitBoard(1152921504606846976),
        BitBoard(9295429630892703744),
        BitBoard(2594073385365405696),
        BitBoard(4755801206503243776),
        BitBoard(71776119061217280),
    ],
];

#[derive(Clone, Copy, PartialEq)]
pub enum Pieces {
    King,
    Queen,
    Rook,
    Bishop,
    Knight,
    Pawn,
    None,
}
impl Pieces {
    pub fn from_u8(index: u8) -> Pieces {
        match index {
            0 => Pieces::King,
            1 => Pieces::Queen,
            2 => Pieces::Rook,
            3 => Pieces::Bishop,
            4 => Pieces::Knight,
            5 => Pieces::Pawn,

            6 => Pieces::None,

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
    for (position, sliding) in table.iter_mut().enumerate().take(BOARD_SIZE) {
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

        *sliding = bitmap.0;
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

impl Default for RaycastTables {
    fn default() -> RaycastTables {
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
