# 🎹 RustWave: An Analog Synth Simulator 🎛️
RustWave aims to provide a realistic emulation of classic analog synthesizer sounds.

## ✨ Features

- 🎚️ Multiple oscillator types (Sine, Square, Sawtooth, Triangle)
- 📊 ADSR envelope generator
- 🔊 Polyphonic voice management
- 🎛️ Ladder filter
- 🖥️ Real-time parameter control via GUI
- ⌨️ QWERTY keyboard input for note playing
- 🖱️ Click-and-drag interface for playing notes

## 🚀 Getting Started

### Prerequisites

- Rust (latest stable version)
- Cargo

### Installation

1. Clone the repository:
   ```
   git clone https://github.com/SauersML/RustWave.git
   cd rustwave
   ```

2. Build the project:
   ```
   cargo build --release
   ```

3. Run RustWave:
   ```
   cargo run --release
   ```

## 🎛️ Usage

Once RustWave is running, you'll see the GUI with various controls:

- Use the sliders to adjust volume, ADSR envelope parameters, and filter settings.
- Select different waveforms.
- Play notes using your computer keyboard (Z-M for lower octave, Q-P for higher octave).
- Click on the on-screen keyboard to play notes with your mouse.

## 🧪 Technical Details

- **Audio Engine**: Uses CPAL (Cross-Platform Audio Library) for low-latency audio output.
- **Oscillators**: Implement polyBLEP anti-aliasing for improved sound quality.
- **Filter**: Moog-inspired ladder filter with resonance and oversampling.
- **Envelope**: ADSR (Attack, Decay, Sustain, Release) envelope with exponential curves.
- **Voice Management**: Polyphonic with voice stealing for optimal performance.
