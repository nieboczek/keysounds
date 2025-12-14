# keysounds
![](examples/showcase.gif)

**keysounds** is a TUI app built in Rust that mixes audio files and your microphone into a virtual output device. It also enables you to filter your microphone to sound like you're in a church or whatever other place else you imagine.

## Features
- **Audio file mixing** - Mixes your microphone input with audio files and outputs to a virtual output device.
- **Audio playback using only your keyboard** - Play audio with configurable global hotkeys.
- **Microphone filtering** - Through filtering you can turn your microphone to one inside a running microwave<sup title="this is a joke btw">[_[citation needed](https://en.wikipedia.org/wiki/Joke)_]</sup>, or you can change your voice to be reverbed, or even bass boosted.
- **Random audio triggering** - Can be enabled to play a random audio from a configurable list every X to Y seconds.

## Installation
Get a prebuilt for your system (Windows only for now) executable on the [Releases](https://github.com/nieboczek/keysounds/releases) tab or compile it yourself by following the steps below.

1. Install Rust and Cargo if not already installed: [https://rust-lang.org/tools/install](https://www.rust-lang.org/tools/install)
2. Clone this repository:
   ```sh
   git clone https://github.com/nieboczek/keysounds.git
   cd keysounds
   ```
3. Build and run:
   ```sh
   cargo run --release
   ```

**Make sure to configure the Virtual Output Device after running keysounds once**

## Configuration
keysounds uses a TOML config file that will be automatically generated next to the executable after the first run. Here are the configuration options:
```toml
# Your microphone device name
input_device = "Microphone (2- Shure MV7)"
# Your virtual output device name
# (Cable for Windows/Mac: https://vb-audio.com/Cable)
output_device = "CABLE Input (VB-Audio Virtual Cable)"
# Random Sfx Triggering time range in seconds
# (currently 133.7s to 420s between audios)
rst_range = [133.7, 420]
# List of audios that can be selected by Random Sfx Triggering
rst_audio_list = ["METAL PIPE", "Moyai 🗿"]

# A keybind object
[[keybinds]]
shift = false # Does shift need to be pressed to trigger
ctrl = true # Does control need to be pressed to trigger
alt = true # Does alt need to be pressed to trigger
key = "KeyT" # Key to trigger action
action = "search_and_play" # Action that will be triggered if the key combination is pressed

[[keybinds]]
shift = false
ctrl = true
alt = true
key = "KeyY"
action = "skip_to_part"

[[keybinds]]
shift = false
ctrl = true
alt = true
key = "KeyS"
action = "stop_sfx"

# An sfx object
[[sfx]]
name = "Dream Speedrun Music" # Unique identifier used in audio search
path = "D:/music/dream_speedrun.mp3" # Path to the audio file
skip_to = 114.2 # (Optional; Default = 0) Position in seconds to skip to
volume = 0.9 # (Optional; Default = 1) Audio volume (1.0 = 100%)

[[sfx]]
name = "Moyai 🗿"
path = "D:/sfx_ogg/moyai.ogg"

[[sfx]]
name = "METAL PIPE"
path = "D:/sfx/metal_pipe.mp3"
volume = 0.69
```

## Contributing
Contributions are welcome!  
Feel free to:
- Open issues for bugs or feature suggestions.
- Submit PRs for fixes or new features.
- Discuss ideas in the issues tab.

## License
This project is licensed under the [GPL v3.0 License](https://github.com/nieboczek/keysounds/blob/master/LICENSE).
