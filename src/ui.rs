use eframe::egui::{self, Color32, Rect, Stroke, Vec2};
use std::sync::Arc;
use parking_lot::Mutex;
use crate::oscillator::Oscillator;

const WHITE_KEYS: [&str; 7] = ["C", "D", "E", "F", "G", "A", "B"];
const BLACK_KEYS: [&str; 5] = ["C#", "D#", "F#", "G#", "A#"];

pub struct SynthUI {
    oscillator: Arc<Mutex<Oscillator>>,
}

impl SynthUI {
    pub fn new(oscillator: Arc<Mutex<Oscillator>>) -> Self {
        Self { oscillator }
    }

    pub fn update(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Synth Keyboard");
            self.draw_keyboard(ui);
        });
    }

    fn draw_keyboard(&mut self, ui: &mut egui::Ui) {
        let available_width = ui.available_width();
        let white_key_width = available_width / 7.0;
        let white_key_height = 120.0;
        let black_key_width = white_key_width * 0.6;
        let black_key_height = white_key_height * 0.6;

        // Draw white keys
        for (i, key) in WHITE_KEYS.iter().enumerate() {
            let rect = Rect::from_min_size(
                ui.min_rect().min + Vec2::new(i as f32 * white_key_width, 0.0),
                Vec2::new(white_key_width, white_key_height),
            );
            if ui.put(rect, egui::Button::new(*key)).clicked() {
                self.play_note(i as u8);
            }
        }

        // Draw black keys
        for (i, key) in BLACK_KEYS.iter().enumerate() {
            let x_offset = match i {
                0 => white_key_width * 0.75,
                1 => white_key_width * 1.75,
                2 => white_key_width * 3.75,
                3 => white_key_width * 4.75,
                4 => white_key_width * 5.75,
                _ => unreachable!(),
            };
            let rect = Rect::from_min_size(
                ui.min_rect().min + Vec2::new(x_offset, 0.0),
                Vec2::new(black_key_width, black_key_height),
            );
            if ui.put(rect, egui::Button::new(*key).fill(Color32::BLACK).stroke(Stroke::new(1.0, Color32::WHITE))).clicked() {
                self.play_note(i as u8 + 1);
            }
        }
    }

    fn play_note(&mut self, note: u8) {
        let frequency = Oscillator::note_to_frequency(note + 60); // Start from middle C (MIDI note 60)
        self.oscillator.lock().set_frequency(frequency);
        println!("Playing note: {} Hz", frequency);
    }
}