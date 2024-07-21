use eframe::egui::{self, Color32, Rect, Stroke, Vec2, Key};
use std::sync::Arc;
use parking_lot::Mutex;
use crate::oscillator::{Oscillator, Waveform};
use crate::voice_manager::VoiceManager;

const OCTAVES: usize = 3;
const WHITE_KEY_INDICES: [usize; 7] = [0, 2, 4, 5, 7, 9, 11];
const BLACK_KEY_INDICES: [usize; 5] = [1, 3, 6, 8, 10];

pub struct SynthUI {
    current_octave: i32,
    key_states: [bool; 128],
    volume: f32,
    waveform: Waveform,
    attack: f32,
    decay: f32,
    sustain: f32,
    release: f32,
    filter_cutoff: f32,
    filter_resonance: f32,
    active_mouse_note: Option<u8>,
    voice_manager: Arc<Mutex<VoiceManager>>,
}

impl SynthUI {
    pub fn new(voice_manager: Arc<Mutex<VoiceManager>>) -> Self {
        Self {
            voice_manager,
            current_octave: 4,
            key_states: [false; 128],
            volume: 0.5,
            waveform: Waveform::Sawtooth,
            attack: 0.1,
            decay: 0.1,
            sustain: 0.7,
            release: 0.2,
            filter_cutoff: 1000.0,
            filter_resonance: 0.0,
            active_mouse_note: None,
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
                self.draw_filter_controls(ui);
                ui.add_space(10.0);
                self.draw_keyboard(ui);
                self.handle_keyboard_input(ctx);
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
                        let mut vm = self.voice_manager.lock();
                        for voice in &mut vm.voices {
                            voice.oscillator.set_volume(self.volume);
                        }
                    }
                });
            });
            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.label("Waveform");
                    for waveform in [Waveform::Sine, Waveform::Square, Waveform::Sawtooth, Waveform::Triangle].iter() {
                        if ui.selectable_value(&mut self.waveform, *waveform, format!("{:?}", waveform)).clicked() {
                            let mut vm = self.voice_manager.lock();
                            for voice in &mut vm.voices {
                                voice.oscillator.set_waveform(self.waveform);
                            }
                        }
                    }
                });
            });
        });
    }

    fn draw_envelope_controls(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let mut attack = self.attack;
            let mut decay = self.decay;
            let mut sustain = self.sustain;
            let mut release = self.release;
    
            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.label("Attack");
                    if ui.add(egui::Slider::new(&mut attack, 0.01..=2.0).logarithmic(true)).changed() {
                        self.attack = attack;
                        let mut vm = self.voice_manager.lock();
                        for voice in &mut vm.voices {
                            voice.envelope.set_attack(self.attack);
                        }
                    }
                });
            });
    
            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.label("Decay");
                    if ui.add(egui::Slider::new(&mut decay, 0.01..=2.0).logarithmic(true)).changed() {
                        self.decay = decay;
                        let mut vm = self.voice_manager.lock();
                        for voice in &mut vm.voices {
                            voice.envelope.set_decay(self.decay);
                        }
                    }
                });
            });
    
            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.label("Sustain");
                    if ui.add(egui::Slider::new(&mut sustain, 0.0..=1.0)).changed() {
                        self.sustain = sustain;
                        let mut vm = self.voice_manager.lock();
                        for voice in &mut vm.voices {
                            voice.envelope.set_sustain(self.sustain);
                        }
                    }
                });
            });
    
            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.label("Release");
                    if ui.add(egui::Slider::new(&mut release, 0.01..=2.0).logarithmic(true)).changed() {
                        self.release = release;
                        let mut vm = self.voice_manager.lock();
                        for voice in &mut vm.voices {
                            voice.envelope.set_release(self.release);
                        }
                    }
                });
            });
        });
    }

    fn draw_filter_controls(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.label("Filter Cutoff");
                    if ui.add(egui::Slider::new(&mut self.filter_cutoff, 20.0..=20000.0).logarithmic(true)).changed() {
                        self.voice_manager.lock().set_filter_cutoff(self.filter_cutoff);
                    }
                });
            });

            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.label("Filter Resonance");
                    if ui.add(egui::Slider::new(&mut self.filter_resonance, 0.0..=4.0)).changed() {
                        self.voice_manager.lock().set_filter_resonance(self.filter_resonance);
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
    
        let (rect, response) = ui.allocate_exact_size(Vec2::new(available_width, white_key_height), egui::Sense::click_and_drag());
        let painter = ui.painter();
    
        self.handle_mouse_input(ui, rect, &response);
    
        // Draw white keys
        for visual_octave in 0..OCTAVES {
            for (i, &key_index) in WHITE_KEY_INDICES.iter().enumerate() {
                if let Some(note) = self.calculate_midi_note(visual_octave as i32, key_index) {
                    let x = (visual_octave * 7 + i) as f32 * white_key_width;
                    let key_rect = Rect::from_min_size(
                        rect.min + Vec2::new(x, 0.0),
                        Vec2::new(white_key_width, white_key_height),
                    );
                    let color = if self.key_states[note as usize] {
                        Color32::LIGHT_BLUE
                    } else {
                        Color32::WHITE
                    };
                    painter.rect_filled(key_rect, 0.0, color);
                    painter.rect_stroke(key_rect, 0.0, Stroke::new(1.0, Color32::BLACK));
                }
            }
        }
    
        // Draw black keys
        for visual_octave in 0..OCTAVES {
            for (i, &key_index) in BLACK_KEY_INDICES.iter().enumerate() {
                if let Some(note) = self.calculate_midi_note(visual_octave as i32, key_index) {
                    let x = match i {
                        0 => white_key_width * 0.75,
                        1 => white_key_width * 1.75,
                        2 => white_key_width * 3.75,
                        3 => white_key_width * 4.75,
                        4 => white_key_width * 5.75,
                        _ => unreachable!(),
                    };
                    let key_rect = Rect::from_min_size(
                        rect.min + Vec2::new(x + visual_octave as f32 * 7.0 * white_key_width, 0.0),
                        Vec2::new(black_key_width, black_key_height),
                    );
                    let color = if self.key_states[note as usize] {
                        Color32::LIGHT_BLUE
                    } else {
                        Color32::BLACK
                    };
                    painter.rect_filled(key_rect, 0.0, color);
                    painter.rect_stroke(key_rect, 0.0, Stroke::new(1.0, Color32::WHITE));
                }
            }
        }
    }



    fn get_note_from_pointer(&self, pos: egui::Pos2, rect: Rect) -> Option<u8> {
        let rel_pos = pos - rect.min;
        let octave_width = rect.width() / (OCTAVES as f32);
        let x_in_keyboard = rel_pos.x;
        let y = rel_pos.y;

        let white_key_width = octave_width / 7.0;
        let black_key_width = white_key_width * 0.6;
        let black_key_height = rect.height() * 0.6;

        // Calculate the visual octave and key within the keyboard
        let visual_octave = (x_in_keyboard / octave_width) as i32;
        let x_in_octave = x_in_keyboard % octave_width;

        // Check black keys first
        for (i, &key_index) in BLACK_KEY_INDICES.iter().enumerate() {
            let x = match i {
                0 => white_key_width * 0.75,
                1 => white_key_width * 1.75,
                2 => white_key_width * 3.75,
                3 => white_key_width * 4.75,
                4 => white_key_width * 5.75,
                _ => unreachable!(),
            };
            if x_in_octave >= x && x_in_octave < x + black_key_width && y < black_key_height {
                return self.calculate_midi_note(visual_octave, key_index);
            }
        }

        // If not a black key, it must be a white key
        let white_key_index = (x_in_octave / white_key_width) as usize;
        if white_key_index < WHITE_KEY_INDICES.len() {
            let key_index = WHITE_KEY_INDICES[white_key_index];
            return self.calculate_midi_note(visual_octave, key_index);
        }

        None
    }




    fn handle_keyboard_input(&mut self, ctx: &egui::Context) {
        const KEYS: [Key; 24] = [
            Key::Z, Key::S, Key::X, Key::D, Key::C, Key::V, Key::G, Key::B, Key::H, Key::N, Key::J, Key::M,
            Key::Q, Key::Num2, Key::W, Key::Num3, Key::E, Key::R, Key::Num5, Key::T, Key::Num6, Key::Y, Key::Num7, Key::U,
        ];
    
        for &key in KEYS.iter() {
            if ctx.input(|i| i.key_pressed(key)) {
                if let Some(note) = self.key_to_note(key) {
                    self.play_note(note);
                }
            }
        }

        // Check for key releases
        for &key in KEYS.iter() {
            if ctx.input(|i| i.key_released(key)) {
                if let Some(note) = self.key_to_note(key) {
                    self.stop_note(note);
                }
            }
        }
    }

    fn handle_mouse_input(&mut self, ui: &egui::Ui, rect: Rect, response: &egui::Response) {
        if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
            if let Some(note) = self.get_note_from_pointer(pos, rect) {
                if response.clicked() || (response.dragged() && Some(note) != self.active_mouse_note) {
                    // Stop the previous note if there was one
                    if let Some(old_note) = self.active_mouse_note.take() {
                        self.stop_note(old_note);
                    }
                    // Play the new note
                    self.play_note(note);
                    self.active_mouse_note = Some(note);
                }
            }
        }

        // Check if the mouse button is released or if the pointer leaves the keyboard area
        if response.drag_released() || (!response.dragged() && self.active_mouse_note.is_some()) {
            if let Some(old_note) = self.active_mouse_note.take() {
                self.stop_note(old_note);
            }
        }
    }

    fn key_to_note(&self, key: Key) -> Option<u8> {
        let base_index = match key {
            Key::Z => 0, Key::S => 1, Key::X => 2, Key::D => 3, Key::C => 4, Key::V => 5,
            Key::G => 6, Key::B => 7, Key::H => 8, Key::N => 9, Key::J => 10, Key::M => 11,
            Key::Q => 12, Key::Num2 => 13, Key::W => 14, Key::Num3 => 15, Key::E => 16, Key::R => 17,
            Key::Num5 => 18, Key::T => 19, Key::Num6 => 20, Key::Y => 21, Key::Num7 => 22, Key::U => 23,
            _ => return None,
        };

        let octave_offset = base_index / 12;
        let note_index = base_index % 12;
        self.calculate_midi_note(octave_offset, note_index.try_into().unwrap())
    }

    fn calculate_midi_note(&self, visual_octave: i32, key_index: usize) -> Option<u8> {
        let base_note = (self.current_octave + visual_octave) * 12 + key_index as i32;
        if base_note >= 0 && base_note <= 127 {
            Some(base_note as u8)
        } else {
            None
        }
    }

    fn play_note(&mut self, note: u8) {
        self.voice_manager.lock().note_on(note);
        self.key_states[note as usize] = true;
        println!("Playing note: {} ({:.2} Hz)", note, Oscillator::note_to_frequency(note));
    }

    fn stop_note(&mut self, note: u8) {
        self.voice_manager.lock().note_off(note);
        self.key_states[note as usize] = false;
        println!("Stopping note: {}", note);
    }
}