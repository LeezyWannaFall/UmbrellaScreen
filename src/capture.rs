use std::{
    io::Write,
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};
pub type Result<T> = std::result::Result<T, String>;

pub fn screens() -> Result<Vec<(String, Vec<u8>)>> {
    let output = Command::new("hyprctl")
        .args(["-j", "monitors"])
        .output()
        .map_err(|e| e.to_string())?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).into_owned());
    }
    let monitors: serde_json::Value =
        serde_json::from_slice(&output.stdout).map_err(|e| e.to_string())?;
    let jobs: Vec<_> = monitors
        .as_array()
        .ok_or("Нет мониторов")?
        .iter()
        .filter_map(|m| m["name"].as_str())
        .map(|name| {
            let name = name.to_owned();
            thread::spawn(move || {
                let output = Command::new("grim")
                    .args(["-l", "0", "-o", &name, "-"])
                    .output()
                    .map_err(|e| format!("grim: {e}"))?;
                if !output.status.success() {
                    return Err(format!("grim: {}", String::from_utf8_lossy(&output.stderr)));
                }
                Ok((name, output.stdout))
            })
        })
        .collect();
    jobs.into_iter()
        .map(|job| job.join().map_err(|_| "Ошибка потока захвата".to_owned())?)
        .collect()
}

pub fn clipboard(png: Vec<u8>) -> Result<()> {
    clipboard_command(png, &mut Command::new("wl-copy"))
}

fn clipboard_command(png: Vec<u8>, command: &mut Command) -> Result<()> {
    // wl-copy's background owner must not inherit our captured stdout/stderr pipes.
    let mut child = command
        .args(["--type", "image/png"])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("Не удалось запустить wl-copy: {e}"))?;
    let mut stdin = child.stdin.take().unwrap();
    let writer = thread::spawn(move || stdin.write_all(&png));
    let start = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break Ok(status),
            Ok(None) if start.elapsed() < Duration::from_secs(20) => {
                thread::sleep(Duration::from_millis(20))
            }
            other => {
                let _ = child.kill();
                let _ = child.wait();
                break Err(format!(
                    "wl-copy: {}",
                    other
                        .err()
                        .map(|e| e.to_string())
                        .unwrap_or("превышено время ожидания".into())
                ));
            }
        }
    };
    let written = writer.join().map_err(|_| "Ошибка передачи PNG")?;
    let status = status?;
    if !status.success() {
        return Err(format!(
            "wl-copy завершился с кодом {:?}; проверьте Wayland-сессию",
            status.code()
        ));
    }
    written.map_err(|e| format!("Не удалось передать PNG: {e}"))
}
