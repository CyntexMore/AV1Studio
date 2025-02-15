# AV1Studio

A GUI for AV1 encoding via Av1an and SVT-AV1-PSY written in Rust using iced.

## Building

To build **AV1Studio**, you only have to do this:

```bash
git clone https://github.com/CyntexMore/AV1Studio.git
cd AV1Studio/
cargo build --release
```

The built binary will be located at `./target/release/av1studio`. You'll want to launch the program from a terminal without a .desktop file, as, for now, **AV1Studio** prints the generated Av1an command to the terminal.

## Bug Reporting

Report bugs at the [GitHub issues page](https://github.com/CyntexMore/AV1Studio/issues).
