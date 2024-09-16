#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)] // it's an example

use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use eframe::egui;
use egui::{vec2, Color32};
use fchess::{
    engine::get_best_move, human_readable_position, structs::Color, Board, BoardState, ChessTables,
};

static SPACING: egui::emath::Vec2 = vec2(1.0, 1.0);
static BUTTON_SIZE: [f32; 2] = [64.0, 64.0];

static ENGINE_DEPTH: usize = 6;
fn ai_player(board_mutex: Arc<Mutex<Board>>) {
    let tables = &ChessTables::default();
    loop {
        thread::sleep(Duration::from_millis(100)); // mitigate busy waiting
        let readonly_board;
        {
            readonly_board = match board_mutex.try_lock() {
                Ok(value) => value.clone(),
                Err(_) => continue,
            }
        }

        if readonly_board.turn != COMPUTER_COLOR {
            continue;
        }
        if readonly_board.get_board_state(tables) != BoardState::OnGoing {
            continue;
        }
        let best_move = get_best_move(ENGINE_DEPTH, readonly_board, tables); // calculate best move while lock is not obtained,

        let mut board;
        {
            board = board_mutex.lock().unwrap();
        }
        if board.get_board_state(tables) == BoardState::OnGoing && board.turn == COMPUTER_COLOR {
            *board = board.move_piece(best_move);
        }
    }
}

static COMPUTER_COLOR: Color = Color::White;
fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default(),
        ..Default::default()
    };

    let tables = ChessTables::default();

    let mut previous_colormap = 0;
    let mut previous_click: Option<u64> = None;
    let mut color_mask = 0;

    let board_mutex = Arc::new(Mutex::new(Board::fen_parser(
        "8/5K2/3Q4/8/8/6k1/8/8 b - - 0 1",
    )));

    let mut text = board_mutex.lock().unwrap().get_text_representation();

    {
        let board_mutex_clone = board_mutex.clone();
        thread::spawn(move || {
            ai_player(board_mutex_clone);
        });
    }

    eframe::run_simple_native("Chess", options, move |ctx, _frame| {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.style_mut().spacing.item_spacing = SPACING;

            ui.style_mut().text_styles.insert(
                egui::TextStyle::Button,
                egui::FontId::new(48.0, eframe::epaint::FontFamily::Proportional),
            );

            let is_lock_open = { board_mutex.try_lock().is_ok() };
            if !is_lock_open {
                for position_index in 0..64 {
                    let _ = egui::Button::new(&text[position_index as usize]);
                }
            }

            {
                text = board_mutex.try_lock().unwrap().get_text_representation();
            }
            for y in 0..8 {
                ui.horizontal(|row_ui| {
                    for x in 0..8 {
                        let position_index = 63 - (y * 8 + x);

                        let position_bitmask: u64 = 1 << position_index;

                        let mut title;

                        if color_mask & position_bitmask != 0 {
                            title = egui::Button::new(&text[position_index as usize])
                                .fill(Color32::GREEN)
                        } else {
                            title = egui::Button::new(&text[position_index as usize])
                        };
                        if let Some(selected_square) = previous_click {
                            if selected_square == position_index {
                                title = egui::Button::new(&text[position_index as usize])
                                    .fill(Color32::YELLOW)
                            }
                        }

                        if row_ui.add_sized(BUTTON_SIZE, title).clicked() {
                            {
                                color_mask = board_mutex
                                    .try_lock()
                                    .unwrap()
                                    .get_legal_movement_mask_safe(position_index as u8, &tables)
                                    .0;
                            }
                            //color_mask = board.get_pseudolegal_move_mask(position_index as u8);

                            if let Some(previous_click_position) = previous_click {
                                {
                                    board_mutex.try_lock().unwrap().try_make_move(
                                        previous_click_position as u8,
                                        position_index as u8,
                                        &tables,
                                    );
                                }
                                println!(
                                    "{}{}",
                                    human_readable_position(previous_click_position as u8),
                                    human_readable_position(position_index as u8)
                                );
                            }

                            if color_mask != 0 {
                                previous_click = Some(position_index);
                                previous_colormap = color_mask;
                            } else {
                                previous_click = None;
                            }
                        };
                    }
                });
            }

            if let Ok(value) = board_mutex.try_lock() {
                match value.get_board_state(&tables) {
                    BoardState::Checkmate => {
                        ui.label(format!("{:?} wins!", value.other_color()));
                    }
                    BoardState::Stalemate => {
                        ui.label("Stalemate!");
                    }
                    BoardState::OnGoing => {}
                }
            }

            ui.label("~Zander");
        });
    })
}
