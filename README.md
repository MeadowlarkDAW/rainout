# ☔ rainout ☔ (Rust Audio IN-OUT)
[![Documentation](https://docs.rs/rainout/badge.svg)](https://docs.rs/rainout)
[![Crates.io](https://img.shields.io/crates/v/rainout.svg)](https://crates.io/crates/rainout)
[![License](https://img.shields.io/crates/l/rainout.svg)](https://github.com/MeadowlarkDAW/rainout/blob/main/COPYRIGHT)

A cross-platform, highly configurable, low-latency, and robust solution for connecting to audio and MIDI devices.

> This crate is experimental and a work in progress. It is not yet ready for any kind of production use.

# Current Progress

| Backend              | Single Audio Device            | Multiple Audio Devices  | MIDI In/Out      | MIDI 2.0 In/Out   |
| -----------------    | -------------                  | ------------------      | --------------   | ---------------   |
| Pipewire (Linux)     | ◻ NYI                          | ◻ NYI                   | ◻ NYI             | ◻ NYI             |
| CoreAudio (MacOS)    | ◻ NYI                          | ◻ NYI                   | ◻ NYI             | ◻ NYI             |
| WASAPI (Windows)     | ⚠️ WIP (Currently output-only) | ❓ Not on roadmap*       | ➖ Not Applicable | ➖ Not Applicable |
| Windows MIDI         | ➖ Not Applicable              | ➖ Not Applicable        | ◻ NYI             | ◻ NYI             |
| ASIO (Windows)       | ◻ NYI                          | ◻ NYI                   | ◻ NYI             | ◻ NYI             |
| Jack (Linux)         | ✔️ Functional                  | ✔️ Functional            | ✔️ Functional     | ◻ NYI             |
| Jack (Windows)       | ✔️ Functional                  | ✔️ Functional            | ✔️ Functional     | ◻ NYI             |
| Jack (MacOS)         | ✔️ Functional                  | ✔️ Functional            | ✔️ Functional     | ◻ NYI             |
| Pulseaudio** (Linux) | ◻ NYI                          | ⛔ Not on roadmap       | ◻ NYI             | ⛔ Not on roadmap  |
| Alsa** (Linux)       | ◻ NYI                          | ⛔ Not on roadmap       | ◻ NYI             | ⛔ Not on roadmap  |

> - "Single Audio Device" means the ability to connect to a single audio device. This audio device can be input-only, output-only, or natively duplex (i.e. an audio interface with both microphone inputs and speaker outputs). "Multiple Audio Devices" means the ability to connect to a separate input and output audio device and then syncing them together in software.
> - \* WASAPI makes it tricky to sync multiple audio devices together without adding a lot of extra latency, which is why it's not currently on the roadmap. However, we may add support for this if there is enough demand for it.
> - ** The Pulseaudio and ALSA backends may prove to be unecessary in the advent of Pipewire.

# Contributing

If you wish to contribute, take a look at the current [Design Document].

If you have any questions, you can reach us in the [Meadowlark Discord Server] under the `rainout` channel or the [Rust Audio Discord Server] under the `rusty-daw` channel.

[Design Document]: ./DESIGN_DOC.md
[Meadowlark Discord Server]: https://discord.gg/2W3Xvc8wy4
[Rust Audio Discord Server]: https://discord.gg/Qs2Zwtf9Gf
