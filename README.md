# rendezvous-coach
This program uses TTS through the [tts](https://docs.rs/tts/latest/tts/index.html) crate.

## Install
To use this in Linux, you need to install:

- [speech-dispatcher](https://wiki.archlinux.org/title/Speech_dispatcher): speech synthesis layer
- [piper-tts-bin](https://github.com/OHF-Voice/piper1-gpl) (AUR): fast and local neural text-to-speech engine
- Then the voices for the languages you want to use (warning: **BIG packages!**):
    - `piper-voices-it-it`
    - `piper-voices-en-us`
    - ...
- `sox`: required to play the audio since speech-dispatcher only supports Piper through a generic module interface using a custom shell command

To configure speech-dispatcher to use Piper, see the [documentation](https://wiki.archlinux.org/title/Speech_dispatcher) (and [this](https://github.com/ken107/read-aloud/issues/375#issuecomment-1937517761)). In short, you need to enable the module in the `~/.config/speech-dispatcher/speechd.conf` file:

```
AddModule "piper" "sd_generic" "piper.conf"
DefaultModule piper
LanguageDefaultModule "it" "piper"
```

And configure the voice to use with the `~/.config/speech-dispatcher/modules/piper.conf` file:

```
DefaultVoice "it/it_IT/paola/medium/it_IT-paola-medium.onnx"

# Specifying a rarely used symbol & big limit so that speech-dispatcher doesn't cut text into chunks:
GenericDelimiters "˨"
GenericMaxChunkLength 1000000

# These lines are important to specify for every language you'll use, otherwise some characters will not work:
GenericLanguage "it" "it" "utf-8"

GenericCmdDependency "sox"
GenericCmdDependency "aplay"

GenericExecuteSynth \
"echo '$DATA' | /usr/bin/piper-tts --model '/usr/share/piper-voices/$VOICE' --output_raw | sox -r 22050 -c 1 -b 16 -e signed-integer -t raw - -t wav - tempo $RATE pitch $PITCH norm | aplay -r 22050 -f S16_LE -t raw -"

GenericRateAdd 1
GenericPitchAdd 1
GenericVolumeAdd 1
GenericRateMultiply 1
GenericPitchMultiply 1000

# Adding all voices we want:
AddVoice "it" "FEMALE1" "it/it_IT/paola/medium/it_IT-paola-medium.onnx"
AddVoice "it" "MALE1" "it/it_IT/riccardo/x_low/it_IT-riccardo-x_low.onnx"
```

If everything works, you should be able to test the TTS capability of your system using the shell:

```bash
spd-say "Ciao!"
```