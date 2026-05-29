# rendezvous-coach

Notifies you with increasing frequency as your departure time approaches. Uses local neural TTS (Italian voice, no cloud).

## Install

```bash
./install.sh
```

Runs tests, builds a release binary, and copies it together with the required shared libraries to `$HOME/bin`. That directory must exist and be in `PATH`.

On **first run**, the TTS model (`vits-piper-it_IT-paola-medium`, ~67 MB) is downloaded automatically to `~/.local/share/rendezvous-coach/models/`. Subsequent runs use the cached model.

### Requirements

- A C++ compiler and `cmake` (needed once to build — provided by `base-devel` on Arch)
- ALSA (`alsa-lib` on Arch) for audio output

No speech-dispatcher, no piper binary, no sox.

### Custom model path

```bash
rendezvous-coach --model-path /path/to/model/dir -r 20:00 -t 00:15
```

The directory must contain `it_IT-paola-medium.onnx`, `tokens.txt`, and `espeak-ng-data/`. Compatible with any `vits-piper-it_IT-paola-*` model from the [sherpa-onnx model repo](https://github.com/k2-fsa/sherpa-onnx/releases/tag/tts-models).

## Usage

```bash
rendezvous-coach -r 20:00 -t 00:15
```

- `-r` / `--rendezvous`: rendezvous time (today, local time)
- `-t` / `--trip`: travel duration

Notifications are spoken with increasing frequency as departure approaches:
- >1h out → every 15 min
- 30–60 min → every 10 min
- 5–30 min → every 5 min
- <5 min → every 1 min
