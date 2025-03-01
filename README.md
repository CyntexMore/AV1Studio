# AV1Studio

**AV1Studio** is a GUI for AV1 encoding via Av1an and SVT-AV1-PSY, written in Rust using egui. If you're allergic to well-written programs and good-looking UIs, this one is for *you*!

## Development

As of now, **AV1Studio** is barely functional and has a lot of bugs.

## Installing

There are a few requirements. Those being:

* Cargo and rustc, of course
* [KosakaIsMe/Av1an-verbosity](https://github.com/KosakaIsMe/Av1an-verbosity) — Thank you, Kosaka! You saved me with this one.
  * Place the built binary into `/usr/local/bin` with the file name `av1an-verbosity`, or provide the full path to Av1an-verbosity from the GUI.
* [FFmpeg](https://ffmpeg.org/download.html)
* [VapourSynth](https://github.com/vapoursynth/vapoursynth/releases)
* [SVT-AV1-PSY](https://github.com/psy-ex/svt-av1-psy)
* mkvmerge
* libbestsource, ffms2 (a part of FFmpeg), l-smash; only of the three has to be installed
* XDG Desktop Portal

The steps to installing **AV1Studio** are the following:

1. Git clone the repository.

```bash
git clone https://github.com/CyntexMore/AV1Studio.git
```

2. Enter the AV1Studio directory, and compile the program.

```bash
cargo build --release
```
or
```bash
cargo install --git https://github.com/CyntexMore/AV1Studio.git
```
(The built binary is now located in `./target`.)

3. (Optionally and for Linux only) Move the built binary to `/usr/local/bin` and make a .desktop file for it.

## License

**AV1Studio** is released under the [AGPL-3.0](https://github.com/CyntexMore/AV1Studio/blob/main/LICENSE) open-source license.
