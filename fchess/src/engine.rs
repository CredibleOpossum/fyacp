use crate::constants::*;
use crate::human_readable_position;
use crate::Board;
use crate::BoardState;
use crate::ChessMove;
use crate::ChessTables;
use crate::Color;
use crate::Pieces;

fn negamax(depth: usize, max_depth: usize, board: Board, tables: &ChessTables) -> i32 {
    match board.get_board_state(tables) {
        BoardState::Checkmate => return -9999 + (depth as i32), // Score checkmates at a higher depth lower, meaning the engine will choose the fastest checkmate.
        BoardState::Stalemate => return 0,                      // Equal position
        BoardState::OnGoing => {}
    }
    if depth == max_depth {
        return evaluate(&board);
    }

    let move_data = board.get_all_legal_moves(tables);

    let mut max = i32::MIN;
    for possible_move in 0..move_data.1 {
        let legal_move = move_data.0[possible_move];
        let score = -negamax(depth + 1, max_depth, board.move_piece(legal_move), tables);
        max = std::cmp::max(max, score);
    }

    max
}

pub fn get_best_move(depth: usize, board: Board, tables: &ChessTables) -> u16 {
    let mut scores = Vec::new();
    let move_data = board.get_all_legal_moves(tables);
    for possible_move in 0..move_data.1 {
        let legal_move = move_data.0[possible_move];
        let new_board = board.move_piece(legal_move);
        scores.push(-negamax(0, depth, new_board, tables));
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

pub fn evaluate(board: &Board) -> i32 {
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

    match board.turn {
        Color::White => white_value - black_value,
        Color::Black => black_value - white_value,
    }
}
