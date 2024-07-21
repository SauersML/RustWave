use eframe::egui::{self, Color32, Rect, Stroke, Vec2};
use std::sync::Arc;
use parking_lot::Mutex;
use crate::oscillator::{Oscillator, Waveform};

const KEYS_IN_OCTAVE: usize = 12;
const OCTAVES: usize = 3;
const WHITE_KEY_INDICES: [usize; 7] = [0, 2, 4, 5, 7, 9, 11];
const BLACK_KEY_INDICES: [usize; 5] = [1, 3, 6, 8, 10];

pub struct SynthUI {
    oscillator: Arc<Mutex<Oscillator>>,
    current_octave: i32,
    volume: f32,
    waveform: Waveform,
    key_states: [bool; KEYS_IN_OCTAVE * OCTAVES],
    attack: f32,
    decay: f32,
    sustain: f32,
    release: f32,
}

impl SynthUI {
    pub fn new(oscillator: Arc<Mutex<Oscillator>>) -> Self {
        Self {
            oscillator,
            current_octave: 4,
            volume: 0.5,
            waveform: Waveform::Sawtooth,
            key_states: [false; KEYS_IN_OCTAVE * OCTAVES],
            attack: 0.1,
            decay: 0.1,
            sustain: 0.7,
            release: 0.2,
        }
    }

    pub fn update(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                self.draw_header(ui);
                ui.add_space(10.0);
                self.draw_controls(ui);
                ui.add_space(10.0);
                self.draw_envelope_controls(ui);
                ui.add_space(10.0);
                self.draw_keyboard(ui);
            });
        });
    }

    fn draw_header(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading("RustSynth");
            ui.add_space(10.0);
            ui.label(format!("Octave: {}", self.current_octave));
            if ui.button("-").clicked() {
                self.current_octave = (self.current_octave - 1).max(0);
            }
            if ui.button("+").clicked() {
                self.current_octave = (self.current_octave + 1).min(8);
            }
        });
    }



    fn draw_controls(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.label("Volume");
                    if ui.add(egui::Slider::new(&mut self.volume, 0.0..=1.0)).changed() {
                        self.oscillator.lock().set_volume(self.volume);
                    }
                });
            });
            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.label("Waveform");
                    if ui.selectable_value(&mut self.waveform, Waveform::Sine, "Sine").clicked() ||
                       ui.selectable_value(&mut self.waveform, Waveform::Square, "Square").clicked() ||
                       ui.selectable_value(&mut self.waveform, Waveform::Sawtooth, "Sawtooth").clicked() ||
                       ui.selectable_value(&mut self.waveform, Waveform::Triangle, "Triangle").clicked() {
                        self.oscillator.lock().set_waveform(self.waveform);
                    }
                });
            });
        });
    }

    fn draw_envelope_controls(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.label("Attack");
                    if ui.add(egui::Slider::new(&mut self.attack, 0.01..=2.0).logarithmic(true)).changed() {
                        self.oscillator.lock().set_attack(self.attack);
                    }
                });
            });
            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.label("Decay");
                    if ui.add(egui::Slider::new(&mut self.decay, 0.01..=2.0).logarithmic(true)).changed() {
                        self.oscillator.lock().set_decay(self.decay);
                    }
                });
            });
            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.label("Sustain");
                    if ui.add(egui::Slider::new(&mut self.sustain, 0.0..=1.0)).changed() {
                        self.oscillator.lock().set_sustain(self.sustain);
                    }
                });
            });
            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.label("Release");
                    if ui.add(egui::Slider::new(&mut self.release, 0.01..=2.0).logarithmic(true)).changed() {
                        self.oscillator.lock().set_release(self.release);
                    }
                });
            });
        });
    }

    fn draw_keyboard(&mut self, ui: &mut egui::Ui) {
        let available_width = ui.available_width();
        let white_key_width = available_width / (7.0 * OCTAVES as f32);
        let white_key_height = 120.0;
        let black_key_width = white_key_width * 0.6;
        let black_key_height = white_key_height * 0.6;
    
        let (rect, _) = ui.allocate_exact_size(Vec2::new(available_width, white_key_height), egui::Sense::click_and_drag());
        let painter = ui.painter();
    
        // Draw and handle white keys
        for octave in 0..OCTAVES {
            for (i, &key_index) in WHITE_KEY_INDICES.iter().enumerate() {
                let key_num = octave * KEYS_IN_OCTAVE + key_index;
                let x = (octave * 7 + i) as f32 * white_key_width;
                let key_rect = Rect::from_min_size(
                    rect.min + Vec2::new(x, 0.0),
                    Vec2::new(white_key_width, white_key_height),
                );
                let color = if self.key_states[key_num] {
                    Color32::LIGHT_BLUE
                } else {
                    Color32::WHITE
                };
                painter.rect_filled(key_rect, 0.0, color);
                painter.rect_stroke(key_rect, 0.0, Stroke::new(1.0, Color32::BLACK));
    
                // Add key press logic here
                if ui.rect_contains_pointer(key_rect) && ui.input(|i| i.pointer.primary_down()) {
                    if !self.key_states[key_num] {
                        self.key_states[key_num] = true;
                        self.play_note(key_num as u8, true);
                    }
                } else if self.key_states[key_num] && !ui.input(|i| i.pointer.primary_down()) {
                    self.key_states[key_num] = false;
                    self.stop_note(key_num as u8);
                }
            }
        }
    
        // Draw and handle black keys
        for octave in 0..OCTAVES {
            for (i, &key_index) in BLACK_KEY_INDICES.iter().enumerate() {
                let key_num = octave * KEYS_IN_OCTAVE + key_index;
                let x = match i {
                    0 => white_key_width * 0.75,
                    1 => white_key_width * 1.75,
                    2 => white_key_width * 3.75,
                    3 => white_key_width * 4.75,
                    4 => white_key_width * 5.75,
                    _ => unreachable!(),
                };
                let key_rect = Rect::from_min_size(
                    rect.min + Vec2::new(x + octave as f32 * 7.0 * white_key_width, 0.0),
                    Vec2::new(black_key_width, black_key_height),
                );
                let color = if self.key_states[key_num] {
                    Color32::LIGHT_BLUE
                } else {
                    Color32::BLACK
                };
                painter.rect_filled(key_rect, 0.0, color);
                painter.rect_stroke(key_rect, 0.0, Stroke::new(1.0, Color32::WHITE));
    
                // Add key press logic here (same as white keys)
                if ui.rect_contains_pointer(key_rect) && ui.input(|i| i.pointer.primary_down()) {
                    if !self.key_states[key_num] {
                        self.key_states[key_num] = true;
                        self.play_note(key_num as u8, true);
                    }
                } else if self.key_states[key_num] && !ui.input(|i| i.pointer.primary_down()) {
                    self.key_states[key_num] = false;
                    self.stop_note(key_num as u8);
                }
            }
        }
    }

    fn play_note(&mut self, note: u8, trigger_envelope: bool) {
        let mut osc = self.oscillator.lock();
        let frequency = Oscillator::note_to_frequency(note + 12 * self.current_octave as u8);
        osc.set_frequency(frequency);
        if trigger_envelope {
            osc.note_on();
        }
        println!("Playing note: {} Hz", frequency);
    }

    fn stop_note(&mut self, note: u8) {
        self.oscillator.lock().note_off();
        println!("Stopping note: {}", note);
    }
}