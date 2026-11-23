//! TokenBuddy UI — Tauri 2 backend.
//!
//! Tray nativa (Win/Linux) + cliente HTTP à API local em 127.0.0.1:8765.

use serde::Deserialize;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, State,
};

const CONFIG_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../../.tools/host-config.json");

#[derive(Deserialize, Clone)]
struct HostConfig {
    #[serde(default = "default_port")]
    http_port: u16,
    #[serde(default)]
    api_key: String,
}

fn default_port() -> u16 {
    8765
}

fn load_config() -> HostConfig {
    std::fs::read_to_string(CONFIG_PATH)
        .ok()
        .and_then(|s| serde_json::from_str::<HostConfig>(&s.trim_start_matches('\u{feff}')).ok())
        .unwrap_or(HostConfig {
            http_port: 8765,
            api_key: String::new(),
        })
}

#[derive(Clone)]
struct AppState {
    config: HostConfig,
}

/// GET /api/status — retorna o snapshot da API local.
#[tauri::command]
async fn get_status(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let url = format!("http://127.0.0.1:{}/api/status", state.config.http_port);
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(4))
        .build()
        .map_err(|e| e.to_string())?;
    let resp = client
        .get(&url)
        .header("X-TokenBuddy-Key", &state.config.api_key)
        .send()
        .await
        .map_err(|e| format!("Falha ao conectar: {e}"))?;
    if !resp.status().is_success() {
        return Err(format!("API respondeu {}", resp.status()));
    }
    let v: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    Ok(v)
}

/// POST /api/navigate — navega para a página 0–4.
#[tauri::command]
async fn navigate(state: State<'_, AppState>, page: u8) -> Result<(), String> {
    if page > 4 {
        return Err("page deve ser 0..=4".into());
    }
    let url = format!("http://127.0.0.1:{}/api/navigate", state.config.http_port);
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(4))
        .build()
        .map_err(|e| e.to_string())?;
    let resp = client
        .post(&url)
        .header("X-TokenBuddy-Key", &state.config.api_key)
        .header("Content-Type", "application/json")
        .body(format!("{{\"page\": {page}}}") )
        .send()
        .await
        .map_err(|e| format!("Falha ao conectar: {e}"))?;
    if !resp.status().is_success() {
        return Err(format!("API respondeu {}", resp.status()));
    }
    Ok(())
}

/// POST /api/expression — muda o estado de expressão.
#[tauri::command]
async fn set_expression(state: State<'_, AppState>, expr: &str, duration_ms: u64) -> Result<(), String> {
    let url = format!("http://127.0.0.1:{}/api/expression", state.config.http_port);
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(4))
        .build()
        .map_err(|e| e.to_string())?;
    let body = serde_json::json!({"state": expr, "duration": duration_ms});
    let resp = client
        .post(&url)
        .header("X-TokenBuddy-Key", &state.config.api_key)
        .header("Content-Type", "application/json")
        .body(body.to_string())
        .send()
        .await
        .map_err(|e| format!("Falha ao conectar: {e}"))?;
    if !resp.status().is_success() {
        return Err(format!("API respondeu {}", resp.status()));
    }
    Ok(())
}

/// POST /api/agents/run — executa um agente com prompt.
#[tauri::command]
async fn run_agent(state: State<'_, AppState>, agent: &str, prompt: &str) -> Result<serde_json::Value, String> {
    let url = format!("http://127.0.0.1:{}/api/agents/run", state.config.http_port);
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(300))
        .build()
        .map_err(|e| e.to_string())?;
    let body = serde_json::json!({"agent": agent, "prompt": prompt});
    let resp = client
        .post(&url)
        .header("X-TokenBuddy-Key", &state.config.api_key)
        .header("Content-Type", "application/json")
        .body(body.to_string())
        .send()
        .await
        .map_err(|e| format!("Falha ao conectar: {e}"))?;
    if !resp.status().is_success() {
        return Err(format!("API respondeu {}", resp.status()));
    }
    let v: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    Ok(v)
}

/// Sair da aplicação.
#[tauri::command]
fn quit(app: tauri::AppHandle) {
    app.exit(0)
}

pub fn run() {
    let config = load_config();

    tauri::Builder::default()
        .manage(AppState { config })
        .setup(|app| {
            // Menu da tray.
            let status_item = MenuItem::with_id(app, "status", "TokenBuddy: Status", true, None::<&str>)?;
            let page0 = MenuItem::with_id(app, "page-0", "Página 0", true, None::<&str>)?;
            let page1 = MenuItem::with_id(app, "page-1", "Página 1", true, None::<&str>)?;
            let page2 = MenuItem::with_id(app, "page-2", "Página 2", true, None::<&str>)?;
            let page3 = MenuItem::with_id(app, "page-3", "Página 3", true, None::<&str>)?;
            let page4 = MenuItem::with_id(app, "page-4", "Página 4", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "Sair", true, None::<&str>)?;

            let menu = Menu::new(app)?;
            #[cfg(target_os = "windows")]
            {
                let hide_item = MenuItem::with_id(app, "hide-window", "Ocultar janela", true, None::<&str>)?;
                menu.append_items(&[
                    &status_item,
                    &page0,
                    &page1,
                    &page2,
                    &page3,
                    &page4,
                    &hide_item,
                    &quit_item,
                ])?;
            }
            #[cfg(not(target_os = "windows"))]
            menu.append_items(&[
                &status_item,
                &page0,
                &page1,
                &page2,
                &page3,
                &page4,
                &quit_item,
            ])?;

            // Tray icon.
            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .show_menu_on_left_click(true)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "status" => {
                        if let Some(win) = app.get_webview_window("main") {
                            let _ = win.show();
                            let _ = win.set_focus();
                        }
                    }
                    #[cfg(target_os = "windows")]
                    "hide-window" => {
                        if let Some(win) = app.get_webview_window("main") {
                            let _ = win.hide();
                        }
                    }
                    "page-0" | "page-1" | "page-2" | "page-3" | "page-4" => {
                        let page: u8 = event.id.as_ref().chars().last().unwrap_or('0').to_digit(10).unwrap_or(0) as u8;
                        let handle = app.clone();
                        std::thread::spawn(move || {
                            let state = handle.state::<AppState>();
                            let config = state.config.clone();
                            tauri::async_runtime::block_on(async move {
                                let url = format!("http://127.0.0.1:{}/api/navigate", config.http_port);
                                let _ = reqwest::Client::new()
                                    .post(&url)
                                    .header("X-TokenBuddy-Key", &config.api_key)
                                    .header("Content-Type", "application/json")
                                    .body(format!("{{\"page\": {page}}}") )
                                    .send()
                                    .await;
                            });
                        });
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })

                .build(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![get_status, navigate, set_expression, run_agent, quit])
        .run(tauri::generate_context!())
        .expect("erro ao iniciar TokenBuddy UI");
}
