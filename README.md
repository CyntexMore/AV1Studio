# AV1Studio

**AV1Studio** is a GUI for AV1 encoding via Av1an and SVT-AV1-PSY, written in Rust using egui. If you're allergic to well-written programs and good-looking UIs, this one is for *you*!

## `dev` TO-DO

List of the things I want to do before merging `dev` with `main`:

* [ ] Complete settings menu with support for saving settings to `./setting.yaml` with `serde` and `serde_yaml`
* [x] Complete the preset saving/loading mechanics with `serde` and `serde_yaml`
* [ ] Switch to asynchronous file dialogs with `rfd`

## Usage

|                           	|      **Default Value**      	|                                                                                                                                                                         **Description**                                                                                                                                                                        	|
|:-------------------------:	|:---------------------------:	|:--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------:	|
|  **Av1an-verbosity Path** 	| (command) `av1an-verbosity` 	|                                                                                                                                                            Full path to the Av1an-verbosity binary.                                                                                                                                                            	|
|       **Input File**      	|             None            	|                                                                                                                                                                Full path to the input MKV file.                                                                                                                                                                	|
|      **Output File**      	|             None            	|                                                                                                                                                                Full path to the output MKV file.                                                                                                                                                               	|
|      **Scenes File**      	|             None            	|                                                                                                                    Full path to a scenes file. (Check out [Trix's Auto Boost Script](https://github.com/trixoniisama/auto-boost-algorithm).)                                                                                                                   	|
|       **Zones File**      	|             None            	|                                                                                      Full path to a file specifying zones within the video with differing encoder settings. (Check out [Trix's Auto Boost Script](https://github.com/trixoniisama/auto-boost-algorithm).)                                                                                      	|
|     **Source Library**    	|          BestSource         	| Method to use for piping exact ranges of frames to the encoder (determines how frames are extracted and sent to the encoder). BestSource is now, supposedly, the best best and most accurate option, but slightly slower than L-SMASH and ffms2. L-SMASH can sometimes fuck up the frame orders completely. ffms2 might corrupt frames on problematic sources. 	|
|   **File Concatenation**  	|           mkvmerge          	|                                                                                                        Method to use for concatenating encoded chunks and audio into output file. If you don't know what you're doing, just go with the default option.                                                                                                        	|
|  **(Output) Resolution**  	|             None            	|                                                                                                                                                            Resolution to resize the output video to.                                                                                                                                                           	|
| **(Output) Pixel Format** 	|         yuv420p10le         	|                                                                                                                  FFmpeg pixel format to use. It's best to go with yuv420p10le (10-bit color format), even if the input video has 8-bit colors.                                                                                                                 	|
|         **Preset**        	|              4              	|                                       Encoding preset to use. A very simple explanation is that you trade quality for encoding speed, the lower you go. Can be set from a range of 0-13. Generally, the sweet spot will be between 2-4-6, of course, depending on how powerful your CPU is, you might want to go higher.                                       	|
|          **CRF**          	|            27.00            	|                                                                     Sets CRF value. A simple explanation is that you trade file size for quality, the lower you go. Can be set from a range of 0-70, can be set in quarter steps (0.25). Generally, the sweet spot will be between 27-23.                                                                      	|
|    **Synthetic Grain**    	|              0              	|                                                                                                                                                 Sets the strength of the synthetic grain applied to the video.                                                                                                                                                 	|
| **Custom Encoder Params** 	|             None            	|                                                                                                                                    Provides SVT-AV1-PSY custom encoder parameters on top of the already included parameters.                                                                                                                                   	|
|    **Thread Affinity**    	|              0              	|                                                                                                           Pin each worker to a specific set of threads of this size. Leaving this option unspecified allows the OS to schedule all processes spawned.                                                                                                          	|
|        **Workers**        	|              0              	|                                        Number of workers to spawn. It's generally recommended, if you have enough RAM, to set this to the total amount of CPU cores you have for better encoding speeds. Leaving this at the default value will allow Av1an to figure out the amount of workers to spawn automatically.                                        	|

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
