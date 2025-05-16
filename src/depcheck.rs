use std::path::Path;
use std::process::Command;

pub fn exists(path: &Path) -> bool {
    let p = Path::new(path);

    if !p.exists() || !p.is_file() {
        return false;
    }

    true
}

pub fn can_run(path: &Path) -> bool {
    // I'm dumb, so there's probably a better way to do this
    Command::new(path)
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false);

    true
}
