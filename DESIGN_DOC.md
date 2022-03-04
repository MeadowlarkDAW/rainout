# rusty-daw-io Design Document

*(note "rusty-daw-io" may not be the final name of this crate)*

# Objective

The goal of this crate is to provide a powerful, cross-platform, highly configurable, low-latency, and robust solution for connecting audio software to audio and MIDI devices.

## Why not contribute to an already existing project like `RTAudio` or `CPAL`?

### RTAudio
- This API is written in a complicated C++ codebase, making it very tricky to bind to other languages such as Rust.
- This project has a poor track record in its stability and ability to gracefully handle errors (not ideal for live audio software).

### CPAL
In short, CPAL is very opinionated, and we have a few deal-breaking issues with its core design.

- CPAL's design does not handle duplex audio devices well. It spawns each input and output stream into separate threads, requiring the developer to sync them together with ring buffers. This is inneficient for most consumer and professional duplex audio devices which already have their inputs and outputs tied into the same stream to reduce latency.
- The API for searching for and configuring audio devices is cumbersome. It returns a list of every possible combination of configurations available with the system's devices. This is not how a user configuring audio settings through a GUI expects this to work.
- CPAL does not have any support for MIDI devices, so we would need to write our own support for it anyway.

Why not just fork `CPAL`?
- To fix these design issues we would pretty much need to rewrite the whole API anyway. Of course we don't have to work completely from scratch. We can still borrow some of the low-level platform specific code in CPAL.

# Goals
- Support for Linux, Mac, and Windows using the following backends: (and maybe Android and iOS in the future, but that is not a gaurantee)
    - Linux
        - [ ] Jack
        - [ ] Pipewire
        - [ ] Alsa (Maybe, depending on how difficult this is. This could be unecessary if Pipewire turns out to be good enough.)
        - [ ] Pulseaudio (Maybe, depending on how difficult this is. This could be unecessary if Pipewire turns out to be good enough.)
    - Mac
        - [ ] CoreAudio
        - [ ] Jack (Maybe, if it is stable enough on Mac.)
    - Window
        - [ ] WASAPI
        - [ ] ASIO (reluctantly)
        - [ ] Jack (Maybe, if it is stable enough on Windows.)
- Scan the available devices on the system, and present configuration options in a format that is intuitive to an end-user configuring devices inside a settings GUI.
- Send all audio and midi streams into a single high-priority thread, taking advantage of native duplex devices when available. (Audio buffers will be presented as de-interlaced `f32` buffers).
- Robust and graceful error handling, especially while the stream is running.
- Easily save and load configurations to/from a config file.
- A system that will try to automatically create a good initial default configuration.

# Later/Maybe Goals
- Support MIDI 2.0 devices
- Support for OSC devices
- C API bindings

# Non-Goals
- No Android and iOS support (for now atleast)
- No support for using multiple backends at the same time (i.e trying to use WASAPI device as an input and an ASIO device as an output). This will just add a whole slew of complexity and stuff that can go wrong.
- No support for tying multiple separate (non-duplexed) audio devices together. We will only support either connecting to a single duplex audio device *or* connecting to a single non-duplex output device.
    - This one is probably controversal, so let me explain the reasoning:
        - Pretty much all modern external audio devices (a setup used by most professionals and pro-sumers) are already duplex.
        - MacOS (and in Linux using JACK or Pipewire) already packages all audio device streams into a single "system-wide duplex device". So this is really only a Windows-specific problem.
        - Tying together multiple non-duplex audio streams requires an intermediate buffer that adds a sometimes unkowable amount of latency.
        - Allowing for multiple separate audio devices adds a lot of complexity to both the settings GUI and the config file, and a lot more that can go wrong.
        - Some modern DAWs like Bitwig already use this "single audio device only" system, so it's not like it's a new concept.
- No support for non-f32 audio streams.
    - There is just no point in my opinion in presenting any other sample format other than `f32` in such an API. These `f32` buffers will just be converted to/from the native sample format that the device wants behind the scenes.

# API Design

TODO
