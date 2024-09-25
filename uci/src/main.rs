use std::collections::HashMap;
use std::{io::Write, net::TcpStream};

use fchess::engine::get_best_move;
use fchess::structs::ChessMove;
use fchess::{human_readable_position, Board};
use fchess::{perft, ChessTables};
use text_io::read;

const OUTPUT_ADDR: &str = "127.0.0.1:2024";

struct Uci(Option<TcpStream>);
impl Uci {
    fn put(&mut self, text: &str) {
        let debug_message = format!("[ENGINE -> GUI] {}\n", text);
        if let Some(stream) = &mut self.0 {
            stream.write_all(debug_message.as_bytes()).unwrap();
        }
        println!("{}", text);
    }
    fn get(&mut self) -> String {
        let text: String = read!("{}\n");
        let debug_message = format!("[GUI -> ENGINE] {}\n", text);
        if let Some(stream) = &mut self.0 {
            stream.write_all(debug_message.as_bytes()).unwrap();
        }
        text
    }
    fn debug(&mut self, text: &str) {
        // Send message to debug handler, not sent to UCI.
        if let Some(stream) = &mut self.0 {
            stream
                .write_all(format!("[DEBUG] {}\n", text).as_bytes())
                .unwrap();
        }
    }
}

const HUMAN_READBLE_SQAURES: [&str; 64] = [
    "H1", "G1", "F1", "E1", "D1", "C1", "B1", "A1", "H2", "G2", "F2", "E2", "D2", "C2", "B2", "A2",
    "H3", "G3", "F3", "E3", "D3", "C3", "B3", "A3", "H4", "G4", "F4", "E4", "D4", "C4", "B4", "A4",
    "H5", "G5", "F5", "E5", "D5", "C5", "B5", "A5", "H6", "G6", "F6", "E6", "D6", "C6", "B6", "A6",
    "H7", "G7", "F7", "E7", "D7", "C7", "B7", "A7", "H8", "G8", "F8", "E8", "D8", "C8", "B8", "A8",
];
fn parse_command(command: &str) -> (u8, u8) {
    let position_first = &command[0..2].to_uppercase();
    let position_second = &command[2..4].to_uppercase();
    let position_first_int = HUMAN_READBLE_SQAURES
        .iter()
        .position(|&r| r == position_first)
        .unwrap();

    let position_second_int = HUMAN_READBLE_SQAURES
        .iter()
        .position(|&r| r == position_second)
        .unwrap();
    (position_first_int as u8, position_second_int as u8)
}

static DEBUGGING: bool = false;
fn main() {
    let mut uci = if DEBUGGING {
        Uci(Some(
            TcpStream::connect(OUTPUT_ADDR).expect("Failed to connect to output server"),
        ))
    } else {
        Uci(None)
    };

    uci.debug("START");

    let tables = ChessTables::default();

    let mut board = Board::default();
    let mut board_history = HashMap::new();

    loop {
        let command = uci.get();
        let command_split = command.split(" ").collect::<Vec<&str>>();
        match command_split[0] {
            "uci" => {
                uci.put("id name Fyacp");
                uci.put("id author Zander");
                uci.put("uciok");
            }
            "isready" => uci.put("readyok"),
            "quit" => break,
            "ucinewgame" => {}

            "go" => match command_split[1] {
                "perft" => {
                    let depth: u8 = command_split[2]
                        .parse()
                        .expect("depth provided wasn't a vaild usize");
                    let results = perft(board.clone(), depth, &tables);

                    let result_string = format!("Nodes searched: {}", results);
                    uci.put(&result_string);
                }
                "wtime" => {
                    let chess_move = ChessMove::unpack(get_best_move(
                        3,
                        board.clone(),
                        board_history.clone(),
                        &tables,
                    ));
                    let suffix = match chess_move.move_type {
                        fchess::structs::MoveType::QueenPromotion => "q",
                        fchess::structs::MoveType::RookPromotion => "r",
                        fchess::structs::MoveType::BishopPromotion => "b",
                        fchess::structs::MoveType::KnightPromotion => "k",
                        _ => "",
                    };
                    for key in board_history.values() {
                        uci.debug(&key.to_string());
                    }
                    uci.put(&format!(
                        "bestmove {}{}{}",
                        human_readable_position(chess_move.origin).to_lowercase(),
                        human_readable_position(chess_move.destination).to_lowercase(),
                        suffix
                    ));
                }
                _ => {}
            },
            "position" => {
                let moves_index = command_split.iter().position(|&r| r == "moves");
                match command_split[1] {
                    "startpos" => {
                        board = Board::default();
                    }
                    "fen" => {
                        let fen = match moves_index {
                            Some(value) => &command_split[2..value],
                            None => &command_split[2..],
                        };
                        board = Board::fen_parser(&fen.join(" "));
                    }
                    _ => panic!(),
                };

                board_history = HashMap::new();
                board_history.insert(board.bitboards, 1);
                if let Some(index) = moves_index {
                    for chess_move in &command_split[index + 1..] {
                        let possible_seen_count = board_history.get(&board.bitboards);
                        match possible_seen_count {
                            Some(value) => board_history.insert(board.bitboards, value + 1),
                            None => board_history.insert(board.bitboards, 1),
                        };
                        let move_data = parse_command(chess_move);

                        let mut promotion_preference = 'q';
                        if chess_move.len() == 5 {
                            promotion_preference = chess_move.chars().nth(4).unwrap();
                        }
                        board.try_make_move(
                            move_data.0,
                            move_data.1,
                            promotion_preference,
                            &tables,
                        );
                    }
                }
            }

            _ => {}
        }
    }

    uci.debug("END");
}
