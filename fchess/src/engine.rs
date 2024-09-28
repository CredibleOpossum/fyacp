use std::collections::HashMap;

use crate::bitboard::BitBoard;
use crate::constants::*;
use crate::move_generation::human_readable_position;
use crate::Board;
use crate::BoardState;
use crate::ChessMove;
use crate::ChessTables;
use crate::Color;
use crate::Pieces;

static LARGE_VALUE_SAFE: i32 = 999_999; // Number that is large enough to overshadow any other number, but not so large it will overflow.

fn negamax(
    depth: usize,
    max_depth: usize,
    board: Board,
    mut move_history: HashMap<[[BitBoard; 6]; 2], u8>,
    mut alpha: i32,
    beta: i32,
    tables: &ChessTables,
) -> i32 {
    match board.get_board_state(tables) {
        BoardState::Checkmate => return -LARGE_VALUE_SAFE + (depth as i32), // Score checkmates at a higher depth lower, meaning the engine will choose the fastest checkmate (or slowest if negative score).
        BoardState::Stalemate => return 0,                                  // Equal position
        BoardState::OnGoing => {}
    }
    if let Some(value) = move_history.get(&board.bitboards) {
        if *value == 2 {
            // We've seen it twice in the history, I'm also seeing it now, so it's three.
            return 0; // Threefold
        }
    }
    if depth == max_depth {
        return evaluate(&board, tables);
    }

    let move_data = board.get_all_legal_moves(tables);

    let mut max_score = i32::MIN;
    for possible_move in 0..move_data.1 {
        let legal_move = move_data.0[possible_move];
        let new_board = board.move_piece(legal_move);
        let possible_seen_count = move_history.get(&new_board.bitboards);
        match possible_seen_count {
            Some(value) => move_history.insert(board.bitboards, value + 1),
            None => move_history.insert(board.bitboards, 1),
        };
        let score = -negamax(
            depth + 1,
            max_depth,
            new_board,
            move_history.clone(),
            -beta, // Flip these values as maximizing player changes.
            -alpha,
            tables,
        );
        max_score = std::cmp::max(max_score, score);

        if score >= beta {
            break;
        }
        if score > alpha {
            alpha = score;
        }
    }

    max_score
}

pub fn get_best_move(
    depth: usize,
    board: Board,
    move_history: HashMap<[[BitBoard; 6]; 2], u8>,
    tables: &ChessTables,
) -> u16 {
    let mut scores = Vec::new();
    let move_data = board.get_all_legal_moves(tables);
    for possible_move in 0..move_data.1 {
        let legal_move = move_data.0[possible_move];
        let new_board = board.move_piece(legal_move);
        scores.push(-negamax(
            0,
            depth,
            new_board,
            move_history.clone(),
            -LARGE_VALUE_SAFE, // Min on maximizing player's turn
            LARGE_VALUE_SAFE,  // Max on maximizing player's turn
            tables,
        ));
    }

    let mut best_score = i32::MIN;
    let mut best_move_index = 0;
    println!("----------------------------");
    for (index, score) in scores.iter().enumerate() {
        let chess_move = ChessMove::unpack(move_data.0[index]);
        let text = format!(
            "{}{}",
            human_readable_position(chess_move.origin),
            human_readable_position(chess_move.destination)
        );
        println!("{}: {}", text, *score);
        if *score > best_score {
            best_score = *score;
            best_move_index = index;
        }
    }
    println!("----------------------------");

    println!("{}", best_move_index);
    move_data.0[best_move_index]
}

pub fn evaluate(board: &Board, tables: &ChessTables) -> i32 {
    let mut white_value = 0;
    white_value += board.bitboards[Color::White as usize][Pieces::Queen as usize].popcnt() as i32
        * QUEEN_VALUE;
    white_value +=
        board.bitboards[Color::White as usize][Pieces::Rook as usize].popcnt() as i32 * ROOK_VALUE;
    white_value += board.bitboards[Color::White as usize][Pieces::Bishop as usize].popcnt() as i32
        * BISHOP_VALUE;
    white_value += board.bitboards[Color::White as usize][Pieces::Knight as usize].popcnt() as i32
        * KNIGHT_VALUE;
    white_value +=
        board.bitboards[Color::White as usize][Pieces::Pawn as usize].popcnt() as i32 * PAWN_VALUE;

    let mut black_value = 0;
    black_value += board.bitboards[Color::Black as usize][Pieces::Queen as usize].popcnt() as i32
        * QUEEN_VALUE;
    black_value +=
        board.bitboards[Color::Black as usize][Pieces::Rook as usize].popcnt() as i32 * ROOK_VALUE;
    black_value += board.bitboards[Color::Black as usize][Pieces::Bishop as usize].popcnt() as i32
        * BISHOP_VALUE;
    black_value += board.bitboards[Color::Black as usize][Pieces::Knight as usize].popcnt() as i32
        * KNIGHT_VALUE;
    black_value +=
        board.bitboards[Color::Black as usize][Pieces::Pawn as usize].popcnt() as i32 * PAWN_VALUE;

    let white_mobility_value =
        board.get_full_capture_mask(Color::White, tables).popcnt() as i32 * MOBILITY_VALUE;
    let black_mobility_value =
        board.get_full_capture_mask(Color::Black, tables).popcnt() as i32 * MOBILITY_VALUE;
    match board.turn {
        Color::White => white_value - black_value + white_mobility_value - black_mobility_value,
        Color::Black => black_value - white_value + black_mobility_value - black_mobility_value,
    }
}
