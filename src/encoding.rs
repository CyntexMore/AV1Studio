use regex::Regex;
use std::process::Command;

use crate::app::AV1Studio;

pub fn parse_av1an_output(
    output: &str,
    encoded_frames: &mut Option<u32>,
    total_frames: &mut Option<u32>,
    fps: &mut Option<f64>,
    eta_time: &mut Option<String>,
) {
    println!("parse_av1an_output called with: {}", output);
    let re = Regex::new(r"(\d+)\s+(\d+)").unwrap();

    for line in output.lines() {
        if let Some(caps) = re.captures(line) {
            *encoded_frames = caps.get(1).and_then(|m| m.as_str().parse().ok());
            *total_frames = caps.get(2).and_then(|m| m.as_str().parse().ok());
            *fps = caps.get(3).and_then(|m| m.as_str().parse().ok());
            *eta_time = caps.get(4).map(|m| m.as_str().to_string());
        }
    }
}

pub fn generate_command(state: &AV1Studio) -> Command {
    let mut cmd = if state.av1an_verbosity_path.is_empty() {
        Command::new("av1an-verbosity")
    } else {
        Command::new(&state.av1an_verbosity_path)
    };

    // Build command arguments
    if !state.input_file.is_empty() {
        cmd.arg("-i").arg(&state.input_file);
    }
    if !state.output_file.is_empty() {
        cmd.arg("-o").arg(&state.output_file);
    }
    if !state.scenes_file.is_empty() {
        cmd.arg("--scenes").arg(&state.scenes_file);
    }
    if !state.zones_file.is_empty() {
        cmd.arg("--zones").arg(&state.zones_file);
    }
    cmd.arg("--verbose-frame-info")
        .arg("--split-method")
        .arg("av-scenechange");

    cmd.arg("-c").arg(if !state.file_concatenation.is_empty() {
        &state.file_concatenation
    } else {
        "mkvmerge"
    });

    cmd.arg("-m")
        .arg(state.source_library.as_str().to_lowercase());

    if !state.width.is_empty() && !state.height.is_empty() {
        let scale = format!(
            "scale={}:{}:flags=bicubic:param0=0:param1=1/2",
            state.width, state.height
        );
        cmd.arg("-f").arg(format!("-vf {}", scale));
    }

    cmd.arg("--pix-format")
        .arg(state.output_pixel_format.as_str())
        .arg("-e")
        .arg("svt-av1");

    if !state.custom_encode_params.is_empty() {
        cmd.arg("-v").arg(&state.custom_encode_params);
    } else {
        let params = format!(
            "--tune 2 --keyint 1 --lp 2 --irefresh-type 2 --crf {} --preset {} --film-grain {} --color-primaries {:?} --transfer-characteristics {:?} --matrix-coefficients {:?} --color-range {:?}",
            state.crf, state.preset, state.synthetic_grain, state.color_primaries, state.transfer_characteristics, state.matrix_coefficients, state.color_range,
        );
        cmd.arg("--force").arg("-v").arg(params);
    }

    cmd.arg("--set-thread-affinity")
        .arg(&state.thread_affinity)
        .arg("-w")
        .arg(&state.workers);

    cmd
}
