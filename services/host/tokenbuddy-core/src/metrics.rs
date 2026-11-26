//! Métricas do host: CPU/RAM via sysinfo; GPU via nvidia-smi quando disponível.
//!
//! Regra do projeto: nunca inventar zero — se a métrica não estiver disponível,
//! o campo simplesmente não existe no resultado.

use serde_json::{json, Value};
use std::process::Command;
use std::time::Duration;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

/// Amostra as métricas de CPU/RAM (e GPU, se nvidia-smi existir).
///
/// O intervalo de amostragem da CPU é ~100ms, como no Python.
pub fn sample_metrics() -> Value {
    let mut sys = sysinfo::System::new_all();
    sys.refresh_cpu_usage(); // primeira leitura zera a base
    std::thread::sleep(Duration::from_millis(100));
    sys.refresh_cpu_usage();

    let cpu: f64 = if !sys.cpus().is_empty() {
        sys.cpus()
            .iter()
            .map(|c| c.cpu_usage())
            .sum::<f32>() as f64
            / sys.cpus().len() as f64
    } else {
        0.0
    };
    let ram_total_mb = sys.total_memory() as f64 / 1_048_576.0;
    let ram_used_mb = sys.used_memory() as f64 / 1_048_576.0;
    let ram_percent = if ram_total_mb > 0.0 {
        (ram_used_mb / ram_total_mb) * 100.0
    } else {
        0.0
    };

    let mut result = json!({
        "type": "telemetry",
        "cpu": cpu,
        "ram": ram_percent,
        "ram_used": ram_used_mb,
        "ram_total": ram_total_mb,
    });

    if let Some(gpu) = sample_gpu() {
        result.as_object_mut().unwrap().insert("gpu".into(), gpu["gpu"].clone());
        result.as_object_mut().unwrap().insert("vram_used".into(), gpu["vram_used"].clone());
        result.as_object_mut().unwrap().insert("vram_total".into(), gpu["vram_total"].clone());
    }

    result
}

/// Consulta nvidia-smi; retorna None quando o binário não existe ou a leitura falha.
fn sample_gpu() -> Option<Value> {
    let exe = which::which("nvidia-smi").ok()?;
    let mut command = Command::new(exe);
    command.args([
        "--query-gpu=utilization.gpu,memory.used,memory.total",
        "--format=csv,noheader,nounits",
    ]);
    #[cfg(windows)]
    command.creation_flags(create_no_window());
    let output = command.output().ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let line = stdout.lines().next()?.to_string();
    let mut parts = line.split(',').map(|s| s.trim().parse::<f64>());
    let (Some(Ok(gpu)), Some(Ok(used)), Some(Ok(total))) = (parts.next(), parts.next(), parts.next()) else {
        return None;
    };
    if !(0.0..=100.0).contains(&gpu) || used < 0.0 || used > total || total <= 0.0 {
        return None;
    }
    Some(json!({"gpu": gpu, "vram_used": used, "vram_total": total}))
}

/// CREATE_NO_WINDOW no Windows; 0 em outros sistemas.
#[cfg(windows)]
fn create_no_window() -> u32 {
    0x0800_0000
}

#[cfg(not(windows))]
fn create_no_window() -> u32 {
    0
}
