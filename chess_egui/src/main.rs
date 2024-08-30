#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)] // it's an example

use eframe::egui;
use egui::{vec2, Color32};
use fchess::Board;

static SPACING: egui::emath::Vec2 = vec2(1.0, 1.0);
static BUTTON_SIZE: [f32; 2] = [64.0, 64.0];

fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default(),
        ..Default::default()
    };

    let mut board = Board::default();

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
                            color_mask = board.get_legal_movement_mask(position_index as u8);

                            if let Some(previous_click_position) = previous_click {
                                board.try_make_move(
                                    previous_click_position as u8,
                                    position_index as u8,
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

            if board.is_in_checkmate() {
                ui.label(format!("{:?} wins!", board.switch_turn()));
                return;
            }
            ui.label("~Zander");
        });
    })
}
