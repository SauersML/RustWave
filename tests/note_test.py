import pygame.midi
import time
import sys

def main():
    # Initialize pygame midi
    pygame.midi.init()
    
    # List available output devices and automatically select the first one
    output_count = pygame.midi.get_count()
    if output_count == 0:
        print("No MIDI output devices found")
        pygame.midi.quit()
        sys.exit(1)
    
    # Look specifically for IAC Driver first, fall back to first available if not found
    output_id = None
    iac_id = None
    print("Available MIDI output devices:")
    
    for i in range(output_count):
        info = pygame.midi.get_device_info(i)
        # info format: (interface, name, is_input, is_output, is_opened)
        if info[3] == 1:  # is_output
            device_name = info[1].decode()
            print(f"{i}: {device_name}")
            
            # Check if this is an IAC Driver
            if "IAC" in device_name and output_id is None:
                output_id = i
                print(f"Selected IAC Driver: {device_name}")
            
            # Keep track of first available output as fallback
            if iac_id is None:
                iac_id = i
    
    # If no IAC Driver found, use the first available output
    if output_id is None and iac_id is not None:
        output_id = iac_id
        info = pygame.midi.get_device_info(iac_id)
        print(f"No IAC Driver found. Using first available output: {info[1].decode()}")
    
    if output_id is None:
        print("No output devices available")
        pygame.midi.quit()
        sys.exit(1)
    
    # Open the selected output device
    try:
        midi_out = pygame.midi.Output(output_id)
        print(f"Successfully opened MIDI device {output_id}")
    except pygame.midi.MidiException as e:
        print(f"Could not open device {output_id}: {e}")
        pygame.midi.quit()
        sys.exit(1)
    
    # Define C major chord (C, E, G)
    c_major = [60, 64, 67]
    # Define A minor chord (A, C, E)
    a_minor = [57, 60, 64]
    # Define F major chord (F, A, C)
    f_major = [53, 57, 60]
    # Define G dominant 7th chord (G, B, D, F)
    g_dominant = [55, 59, 62, 65]
    # Chord progression
    progression = [c_major, a_minor, f_major, g_dominant]
    
    try:
        print("Playing MIDI notes continuously. Press Ctrl+C to stop.")
        # Play a C major scale first
        for note in range(60, 73):  # C4 to C5
            midi_out.note_on(note, 80)  # Note on, velocity 80
            time.sleep(0.2)
            midi_out.note_off(note, 0)  # Note off
            time.sleep(0.05)
        
        # Then loop through chord progression continuously
        while True:
            for chord in progression:
                # Play each chord
                for note in chord:
                    midi_out.note_on(note, 80)
                time.sleep(1.0)  # Hold for 1 second
                # Release all notes
                for note in chord:
                    midi_out.note_off(note, 0)
                time.sleep(0.2)  # Brief pause between chords
            # Brief pause between progression repeats
            time.sleep(0.5)
    except KeyboardInterrupt:
        print("\nStopped by user")
    finally:
        # Make sure to turn off any hanging notes
        for i in range(128):
            midi_out.note_off(i, 0)
        # Close MIDI and quit pygame
        del midi_out
        pygame.midi.quit()

if __name__ == "__main__":
    main()
