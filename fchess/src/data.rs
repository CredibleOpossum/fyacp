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

fn calculate_sliding(direction: [i32; 2]) -> [u64; 64] {
    let mut table = [0; 64];
    for position in 0..BOARD_SIZE {
        let mut bitmap = BitBoard(0);

        let mut current_position = [position as i32 % 8, position as i32 / 8];

        loop {
            current_position[0] += direction[0];
            current_position[1] += direction[1];

            let vaild_position =
                (0..8).contains(&current_position[0]) && (0..8).contains(&current_position[1]);

            if vaild_position {
                bitmap.set_bit(((current_position[0] % 8) + current_position[1] * 8) as u8);
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
    Blank,
}
