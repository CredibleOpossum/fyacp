#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)] // it's an example

use colored::Colorize;
use eframe::egui;
use egui::{vec2, Color32};
use fchess::{human_readable_position, Board, BoardState, ChessTables};

static SPACING: egui::emath::Vec2 = vec2(1.0, 1.0);
static BUTTON_SIZE: [f32; 2] = [64.0, 64.0];

#[derive(Clone, Copy)]
pub struct BitBoard(pub u64);

impl BitBoard {
    pub fn set_bit(&mut self, bit_index: u8) {
        self.0 |= 1 << bit_index;
    }
    pub fn clear_bit(&mut self, bit_index: u8) {
        self.0 &= u64::MAX ^ (1 << bit_index);
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

fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default(),
        ..Default::default()
    };

    let tables = ChessTables::default();
    let mut board = Board::fen_parser("8/7r/2pp4/KPQ5/5p1k/8/4P1P1/8 w - - 2 3");

    let mut previous_colormap = 0;
    let mut previous_click: Option<u64> = None;
    let mut color_mask = 0;

    eframe::run_simple_native("Chess", options, move |ctx, _frame| {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.style_mut().spacing.item_spacing = SPACING;

            ui.style_mut().text_styles.insert(
                egui::TextStyle::Button,
                egui::FontId::new(48.0, eframe::epaint::FontFamily::Proportional),
            );

            let text = board.get_text_representation();

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
                            color_mask =
                                board.get_legal_movement_mask(position_index as u8, &tables);
                            //color_mask = board.get_pseudolegal_move_mask(position_index as u8);

                            if let Some(previous_click_position) = previous_click {
                                board.try_make_move(
                                    previous_click_position as u8,
                                    position_index as u8,
                                    &tables,
                                );
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

            //match board.get_board_state(&tables) {
            //    BoardState::Checkmate => {
            //        ui.label(format!("{:?} wins!", board.other_color()));
            //    }
            //    BoardState::Stalemate => {
            //        ui.label("Stalemate!");
            //    }
            //    BoardState::OnGoing => {}
            //}
            ui.label("~Zander");
        });
    })
}
