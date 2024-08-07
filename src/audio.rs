use std::process::Command;

pub fn set_audio(level: f32) {
    let level = level.clamp(0.0, 100.0) / 100.0;
    Command::new("wpctl")
        .args(["set-volume", "@DEFAULT_SINK@", &format!("{level}")])
        .spawn()
        .unwrap();
}
