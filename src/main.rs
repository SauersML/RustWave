mod envelope;
mod oscillator;
mod ui;
mod voice;
mod voice_manager;
mod filter;
mod reverb;
mod chorus;
mod midi_handler;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Sample, SampleFormat, SizedSample};
use dasp_sample::FromSample;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use parking_lot::Mutex;
use eframe::egui;

use crate::voice_manager::VoiceManager;
use crate::ui::SynthUI;
use crate::midi_handler::MidiHandler;

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
    let (mut midi_handler, _midi_rx) = MidiHandler::new()?;
    midi_handler.set_voice_manager(Arc::clone(&voice_manager));
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
        let (left, right) = voice_manager.lock().render_next();
        let left_sample = T::from_sample(left);
        let right_sample = T::from_sample(right);

        for (i, sample) in frame.iter_mut().enumerate() {
            *sample = if i % 2 == 0 { left_sample } else { right_sample };
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let host = cpal::default_host();
    let device = host.default_output_device().expect("no output device available");

    println!("Output device: {}", device.name()?);

    // Get all supported configs and find the best one to use
    
    // Preferred formats in order (most preferred first)
    let preferred_formats = [
        (SampleFormat::F32, 48000),
        (SampleFormat::I16, 48000),
        (SampleFormat::F32, 44100),
        (SampleFormat::I16, 44100),
    ];
    
    // Find the best config
    let mut selected_config = None;
    let mut fallback_config = None;

    // First try to find one of our preferred configs
    for supported_config in device.supported_output_configs().expect("error querying configs") {
        // Save the first config as a fallback
        if fallback_config.is_none() {
            fallback_config = Some(supported_config.clone());
        }
        
        let format = supported_config.sample_format();
        let min_rate = supported_config.min_sample_rate().0;
        let max_rate = supported_config.max_sample_rate().0;
        
        // Check if this config matches any of our preferred formats
        for &(preferred_format, preferred_rate) in &preferred_formats {
            if format == preferred_format && 
               min_rate <= preferred_rate && max_rate >= preferred_rate {
                // Found a match with one of our preferred configs
                selected_config = Some(supported_config.with_sample_rate(cpal::SampleRate(preferred_rate)));
                break;
            }
        }
        
        if selected_config.is_some() {
            break;
        }
    }
    
    // Use fallback if no preferred config found
    let supported_config = selected_config.unwrap_or_else(|| {
        fallback_config.expect("no supported config found").with_max_sample_rate()
    });
    
    println!("Selected output config: {:?}", supported_config);
    
    let sample_format = supported_config.sample_format();
    let config: cpal::StreamConfig = supported_config.into();

    match sample_format {
        SampleFormat::F32 => run::<f32>(&device, &config)?,
        SampleFormat::I16 => run::<i16>(&device, &config)?,
        SampleFormat::U16 => run::<u16>(&device, &config)?,
        SampleFormat::U8 => run::<u8>(&device, &config)?,
        SampleFormat::I8 => run::<i8>(&device, &config)?,
        _ => {
            println!("Unsupported sample format: {:?}, trying to use a different format...", sample_format);
            
            // Try to find a supported format
            let mut configs = device.supported_output_configs()
                .expect("error while querying configs");
            
            while let Some(config) = configs.next() {
                let format = config.sample_format();
                if format == SampleFormat::F32 || format == SampleFormat::I16 || 
                   format == SampleFormat::U16 || format == SampleFormat::U8 || 
                   format == SampleFormat::I8 {
                    let stream_config = config.with_max_sample_rate().into();
                    println!("Trying alternative config: {:?}", config);
                    
                    match format {
                        SampleFormat::F32 => return run::<f32>(&device, &stream_config),
                        SampleFormat::I16 => return run::<i16>(&device, &stream_config),
                        SampleFormat::U16 => return run::<u16>(&device, &stream_config),
                        SampleFormat::U8 => return run::<u8>(&device, &stream_config),
                        SampleFormat::I8 => return run::<i8>(&device, &stream_config),
                        _ => continue,
                    }
                }
            }
            
            panic!("Could not find any usable audio configuration");
        }
    }

    Ok(())
}
