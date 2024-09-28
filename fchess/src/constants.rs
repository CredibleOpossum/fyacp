use crate::bitboard::BitBoard;

pub const QUEEN_VALUE: i32 = 1000;
pub const ROOK_VALUE: i32 = 500;
pub const BISHOP_VALUE: i32 = 350;
pub const KNIGHT_VALUE: i32 = 300;
pub const PAWN_VALUE: i32 = 100;
pub const MOBILITY_VALUE: i32 = 2; // PLACEHOLDER

pub const MAX_LEGAL_MOVES: usize = 32;

pub const MAX_MOVE_BUFFER: usize = 256;

pub const _EMPTY: u64 = 0;
pub const _UNIVERSE: u64 = u64::MAX;

pub const BOARD_SIZE: usize = 64;

pub const BOARD_TOP: BitBoard = BitBoard(0xFF00000000000000);
pub const BOARD_BOTTOM: BitBoard = BitBoard(0xFF);
pub const BOARD_LEFT: BitBoard = BitBoard(0x8080808080808080);
pub const BOARD_RIGHT: BitBoard = BitBoard(0x101010101010101);

pub const THICK_BOARD_TOP: BitBoard = BitBoard(0xFFFF000000000000);
pub const THICK_BOARD_BOTTOM: BitBoard = BitBoard(0xFFFF);
pub const THICK_BOARD_LEFT: BitBoard = BitBoard(0xC0C0C0C0C0C0C0C0);
pub const THICK_BOARD_RIGHT: BitBoard = BitBoard(0x303030303030303);

pub const TOP: BitBoard = BitBoard(0xff00000000000000);
pub const LEFT: BitBoard = BitBoard(0x8080808080808080);
pub const RIGHT: BitBoard = BitBoard(0x101010101010101);
pub const BOTTOM: BitBoard = BitBoard(0xff);

pub const EMPTY_STRING: String = String::new();

pub const STARTING_POSITION_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

#[rustfmt::skip]
pub const HUMAN_READBLE_SQAURES: [&str; 64] = [ "H1", "G1", "F1", "E1", "D1", "C1", "B1", "A1",
                                                "H2", "G2", "F2", "E2", "D2", "C2", "B2", "A2",
                                                "H3", "G3", "F3", "E3", "D3", "C3", "B3", "A3",
                                                "H4", "G4", "F4", "E4", "D4", "C4", "B4", "A4",
                                                "H5", "G5", "F5", "E5", "D5", "C5", "B5", "A5",
                                                "H6", "G6", "F6", "E6", "D6", "C6", "B6", "A6",
                                                "H7", "G7", "F7", "E7", "D7", "C7", "B7", "A7",
                                                "H8", "G8", "F8", "E8", "D8", "C8", "B8", "A8"];
