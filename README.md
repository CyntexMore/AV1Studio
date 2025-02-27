# AV1Studio

**AV1Studio** is a GUI for AV1 encoding via Av1an and SVT-AV1-PSY, written in Rust using egui. If you're allergic to well-written programs and good-looking UIs, this one is for *you*!

## Development

As of now, **AV1Studio** has *no* functionality, at all.

## Installing

There are a few requirements. Those being:

* Cargo and rustc, of course
* [Av1an](https://github.com/master-of-zen/Av1an)

The steps to installing **AV1Studio** are the following:

1. Git clone the repository.

```bash
git clone https://github.com/CyntexMore/AV1Studio.git
```

2. Enter the AV1Studio directory, and compile the program.

```bash
cargo build --released
```
(The built binaries are now located in `./target`.)

3. (Optionally and for Linux only) Move the built binaries to `/usr/local/bin` and make a .desktop file for it.

## License

**AV1Studio** is released under the [AGPL-3.0](https://github.com/CyntexMore/AV1Studio/blob/main/LICENSE) open-source license.
