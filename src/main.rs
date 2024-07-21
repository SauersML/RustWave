mod envelope;
mod oscillator;
mod ui;
mod voice;
mod voice_manager;
mod filter;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Sample, SampleFormat, SizedSample};
use dasp_sample::FromSample;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use parking_lot::Mutex;
use eframe::egui;

use crate::voice_manager::VoiceManager;
use crate::ui::SynthUI;

impl eframe::App for SynthApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.ui.update(ctx);
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.running.store(false, Ordering::SeqCst);
    }
}

struct SynthApp {
    ui: SynthUI,
    _stream: cpal::Stream,
    running: Arc<AtomicBool>,
}

fn run<T>(device: &cpal::Device, config: &cpal::StreamConfig) -> Result<(), Box<dyn std::error::Error>>
where
    T: Sample + SizedSample + FromSample<f32>,
{
    let sample_rate = config.sample_rate.0 as f32;
    let channels = config.channels as usize;

    let voice_manager = Arc::new(Mutex::new(VoiceManager::new(sample_rate, 8))); // 8 voices
    let running = Arc::new(AtomicBool::new(true));

    let vm_clone = Arc::clone(&voice_manager);

    let stream = device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            write_data(data, channels, &vm_clone)
        },
        |err| eprintln!("an error occurred on stream: {}", err),
        None,
    )?;

    stream.play()?;

    let ui = SynthUI::new(Arc::clone(&voice_manager));

    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::Vec2::new(1200.0, 800.0)),
        ..Default::default()
    };

    eframe::run_native(
        "Rust Synth",
        options,
        Box::new(|_cc| Box::new(SynthApp { ui, _stream: stream, running })),
    ).map_err(|e| e.to_string())?;

    Ok(())
}

fn write_data<T>(output: &mut [T], channels: usize, voice_manager: &Arc<Mutex<VoiceManager>>)
where
    T: Sample + FromSample<f32>,
{
    for frame in output.chunks_mut(channels) {
        let value = voice_manager.lock().render_next();
        let value = T::from_sample(value);
        for sample in frame.iter_mut() {
            *sample = value;
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let host = cpal::default_host();
    let device = host.default_output_device().expect("no output device available");

    println!("Output device: {}", device.name()?);

    let mut supported_configs_range = device.supported_output_configs().expect("error while querying configs");
    let supported_config = supported_configs_range.next().expect("no supported config?!").with_max_sample_rate();

    println!("Default output config: {:?}", supported_config);

    let sample_format = supported_config.sample_format();
    let config: cpal::StreamConfig = supported_config.into();

    match sample_format {
        SampleFormat::F32 => run::<f32>(&device, &config)?,
        SampleFormat::I16 => run::<i16>(&device, &config)?,
        SampleFormat::U16 => run::<u16>(&device, &config)?,
        _ => panic!("Unsupported sample format: {:?}", sample_format),
    }

    Ok(())
}