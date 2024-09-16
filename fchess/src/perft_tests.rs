#[cfg(test)]
use crate::human_readable_position;
#[cfg(test)]
use crate::Board;
#[cfg(test)]
use crate::BoardState;
#[cfg(test)]
use crate::ChessMove;
#[cfg(test)]
use crate::ChessTables;

#[cfg(test)]
fn perft_internal(board: Board, depth: u8, max_depth: u8, tables: &ChessTables) -> usize {
    let all_legal_moves = board.get_all_legal_moves(tables);
    if depth == max_depth {
        return all_legal_moves.1;
    }

    match board.get_board_state(tables) {
        BoardState::Checkmate => return all_legal_moves.1,
        BoardState::Stalemate => return all_legal_moves.1,
        BoardState::OnGoing => {}
    }

    let mut move_sum = 0;

    for possible_move in 0..all_legal_moves.1 {
        let postmove = board.move_piece(all_legal_moves.0[possible_move]);
        move_sum += perft_internal(postmove, depth + 1, max_depth, tables);
    }

    move_sum
}

#[cfg(test)]
fn perft(board: Board, depth: u8, tables: &ChessTables) -> usize {
    let mut results = Vec::new();

    let mut sum = 0;
    let legal_moves = board.get_all_legal_moves(tables);

    for possible_move in 0..legal_moves.1 {
        let move_count = if depth == 1 {
            1
        } else {
            perft_internal(
                board.move_piece(legal_moves.0[possible_move]),
                1,
                depth - 1,
                tables,
            )
        };
        sum += move_count;
        let parsed = ChessMove::unpack(legal_moves.0[possible_move]);

        results.push(format!(
            "{}{}: {}",
            human_readable_position(parsed.origin),
            human_readable_position(parsed.destination),
            move_count
        ))
    }

    results.sort();
    for result in results {
        println!("{}", result);
    }

    sum
}

#[cfg(test)]
mod tests {
    use crate::{Board, ChessTables, STARTING_POSITION_FEN};

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
        let board = Board::fen_parser(STARTING_POSITION_FEN);
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
