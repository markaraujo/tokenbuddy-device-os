//! TokenBuddy local host (Rust): bridge USB serial + API HTTP local.
//!
//! Porta exata do `tokenbuddy` Python: mesma API, mesmo comportamento de
//! validação e mesmas mensagens de erro.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use serde_json::{json, Value};
use tokenbuddy_core::bridge::{Bridge, BridgeError};
use tokenbuddy_core::connectors::{run_codex, run_command, run_ollama};
use tokenbuddy_core::metrics::sample_metrics;

const VERSION: &str = "0.3.0";

/// Estados de expressão aceitos pelo firmware (mesma ordem de `behavior.h`).
pub const STATES: &[(&str, i32)] = &[
    ("idle", 0),
    ("happy", 1),
    ("thinking", 2),
    ("error", 3),
    ("curious", 4),
    ("working", 5),
    ("success", 6),
    ("sleeping", 7),
    ("disconnected", 8),
    ("dizzy", 9),
];

#[derive(Clone)]
struct AppState {
    bridge: Arc<Mutex<Bridge>>,
    key: String,
}

fn error_response(status: StatusCode, message: &str) -> Response {
    let body = message.to_string();
    (status, body).into_response()
}

/// Valida o header `X-TokenBuddy-Key` (comparação constante).
fn authorized(state: &AppState, headers: &axum::http::HeaderMap) -> Result<(), Response> {
    let provided = headers.get("x-tokenbuddy-key").and_then(|v| v.to_str().ok());
    match provided {
        Some(k) if k.len() == state.key.len() => {
            let mut diff = 0u8;
            for (a, b) in k.bytes().zip(state.key.bytes()) {
                diff |= a ^ b;
            }
            if diff == 0 {
                Ok(())
            } else {
                Err(error_response(StatusCode::UNAUTHORIZED, "Chave local obrigatoria"))
            }
        }
        _ => Err(error_response(StatusCode::UNAUTHORIZED, "Chave local obrigatoria")),
    }
}

/// Converte `BridgeError` em HTTP status, como no Python:
/// - `CommandError` → 409
/// - `ConnectionError`/`TimeoutError`/`OSError` → 503
fn bridge_error_to_response(e: BridgeError) -> Response {
    match e {
        BridgeError::Command(m) => error_response(StatusCode::CONFLICT, &m),
        _ => error_response(StatusCode::SERVICE_UNAVAILABLE, &e.to_string()),
    }
}

async fn health() -> Json<Value> {
    Json(json!({"status": "ok", "version": VERSION}))
}

async fn status(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> Result<Json<Value>, Response> {
    authorized(&state, &headers)?;
    let bridge = state.bridge.lock().unwrap();
    Ok(Json(bridge.snapshot()))
}

/// Valida o payload de evento de agente (regras do Pydantic `AgentEvent`).
fn validate_agent_event(value: &Value) -> Result<Value, Response> {
    let obj = value
        .as_object()
        .ok_or_else(|| error_response(StatusCode::UNPROCESSABLE_ENTITY, "Payload deve ser um objeto"))?;

    // extra="forbid"
    for key in obj.keys() {
        if !["event", "agent", "input_tokens", "output_tokens", "task_id"].contains(&key.as_str()) {
            return Err(error_response(
                StatusCode::UNPROCESSABLE_ENTITY,
                &format!("Campo nao permitido: {key}"),
            ));
        }
    }

    let event = obj.get("event").and_then(|v| v.as_str()).ok_or_else(|| {
        error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "Campo obrigatorio: event",
        )
    })?;
    if !["agent.started", "agent.completed", "agent.failed"].contains(&event) {
        return Err(error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "Event invalido",
        ));
    }

    let agent = obj.get("agent").and_then(|v| v.as_str()).ok_or_else(|| {
        error_response(StatusCode::UNPROCESSABLE_ENTITY, "Campo obrigatorio: agent")
    })?;
    if agent.is_empty() || agent.len() > 31 {
        return Err(error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "agent deve ter 1..=31 caracteres",
        ));
    }

    let mut out = json!({"event": event, "agent": agent});
    let mut input_tokens: Option<i64> = None;
    let mut output_tokens: Option<i64> = None;

    if let Some(v) = obj.get("input_tokens") {
        if !v.is_null() {
            let n = v.as_u64().ok_or_else(|| {
                error_response(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "input_tokens deve ser inteiro >= 0",
                )
            })?;
            if n > 2_147_483_647 {
                return Err(error_response(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "input_tokens excede o limite",
                ));
            }
            input_tokens = Some(n as i64);
        }
    }
    if let Some(v) = obj.get("output_tokens") {
        if !v.is_null() {
            let n = v.as_u64().ok_or_else(|| {
                error_response(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "output_tokens deve ser inteiro >= 0",
                )
            })?;
            if n > 2_147_483_647 {
                return Err(error_response(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "output_tokens excede o limite",
                ));
            }
            output_tokens = Some(n as i64);
        }
    }

    // Regra do endpoint: entrada e saida devem vir juntas.
    if (input_tokens.is_none()) != (output_tokens.is_none()) {
        return Err(error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "Entrada e saida devem ser fornecidas juntas",
        ));
    }
    if let Some(i) = input_tokens {
        out["input_tokens"] = json!(i);
    }
    if let Some(o) = output_tokens {
        out["output_tokens"] = json!(o);
    }

    if let Some(v) = obj.get("task_id") {
        if !v.is_null() {
            let t = v.as_str().ok_or_else(|| {
                error_response(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "task_id deve ser string",
                )
            })?;
            if t.is_empty() || t.len() > 64 || !t.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') {
                return Err(error_response(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "task_id invalido",
                ));
            }
            out["task_id"] = json!(t);
        }
    }

    Ok(merge(json!({"type": "event"}), out))
}

fn merge(mut base: Value, extra: Value) -> Value {
    if let (Some(b), Some(e)) = (base.as_object_mut(), extra.as_object()) {
        for (k, v) in e {
            b.insert(k.clone(), v.clone());
        }
    }
    base
}

async fn event(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Json(value): Json<Value>,
) -> Result<Json<Value>, Response> {
    authorized(&state, &headers)?;
    let message = validate_agent_event(&value)?;
    let bridge = state.bridge.lock().unwrap();
    match bridge.request(message) {
        Ok(ack) => Ok(Json(ack)),
        Err(e) => Err(bridge_error_to_response(e)),
    }
}

/// Valida o payload de settings (regras do Pydantic `Settings`).
fn validate_settings(value: &Value) -> Result<Value, Response> {
    let obj = value.as_object().ok_or_else(|| {
        error_response(StatusCode::UNPROCESSABLE_ENTITY, "Payload deve ser um objeto")
    })?;

    for key in obj.keys() {
        if !["brightness", "animation", "demo"].contains(&key.as_str()) {
            return Err(error_response(
                StatusCode::UNPROCESSABLE_ENTITY,
                &format!("Campo nao permitido: {key}"),
            ));
        }
    }

    let mut out = json!({"type": "settings"});
    if let Some(v) = obj.get("brightness") {
        if !v.is_null() {
            let n = v.as_u64().ok_or_else(|| {
                error_response(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "brightness deve ser inteiro 0..=50",
                )
            })?;
            if n > 50 {
                return Err(error_response(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "brightness excede o limite",
                ));
            }
            out["brightness"] = json!(n);
        }
    }
    if let Some(v) = obj.get("animation") {
        if !v.is_null() {
            let n = v.as_u64().ok_or_else(|| {
                error_response(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "animation deve ser inteiro 0..=2",
                )
            })?;
            if n > 2 {
                return Err(error_response(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "animation excede o limite",
                ));
            }
            out["animation"] = json!(n);
        }
    }
    if let Some(v) = obj.get("demo") {
        if !v.is_null() {
            let b = v.as_bool().ok_or_else(|| {
                error_response(StatusCode::UNPROCESSABLE_ENTITY, "demo deve ser booleano")
            })?;
            out["demo"] = json!(b);
        }
    }

    Ok(out)
}

async fn settings(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Json(value): Json<Value>,
) -> Result<Json<Value>, Response> {
    authorized(&state, &headers)?;
    let message = validate_settings(&value)?;
    let bridge = state.bridge.lock().unwrap();
    match bridge.request(message) {
        Ok(ack) => Ok(Json(ack)),
        Err(e) => Err(bridge_error_to_response(e)),
    }
}

/// Valida o payload de navegação (regras do Pydantic `Navigate`).
fn validate_navigate(value: &Value) -> Result<Value, Response> {
    let obj = value.as_object().ok_or_else(|| {
        error_response(StatusCode::UNPROCESSABLE_ENTITY, "Payload deve ser um objeto")
    })?;

    for key in obj.keys() {
        if key != "page" {
            return Err(error_response(
                StatusCode::UNPROCESSABLE_ENTITY,
                &format!("Campo nao permitido: {key}"),
            ));
        }
    }

    let page = obj.get("page").and_then(|v| v.as_u64()).ok_or_else(|| {
        error_response(StatusCode::UNPROCESSABLE_ENTITY, "page deve ser inteiro 0..=4")
    })?;
    if page > 4 {
        return Err(error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "page excede o limite",
        ));
    }

    Ok(json!({"type": "navigate", "page": page}))
}

async fn navigate(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Json(value): Json<Value>,
) -> Result<Json<Value>, Response> {
    authorized(&state, &headers)?;
    let message = validate_navigate(&value)?;
    let bridge = state.bridge.lock().unwrap();
    match bridge.request(message) {
        Ok(ack) => Ok(Json(ack)),
        Err(e) => Err(bridge_error_to_response(e)),
    }
}

/// Valida o payload de expressão (regras do Pydantic `Expression`).
fn validate_expression(value: &Value) -> Result<Value, Response> {
    let obj = value.as_object().ok_or_else(|| {
        error_response(StatusCode::UNPROCESSABLE_ENTITY, "Payload deve ser um objeto")
    })?;

    for key in obj.keys() {
        if !["state", "duration"].contains(&key.as_str()) {
            return Err(error_response(
                StatusCode::UNPROCESSABLE_ENTITY,
                &format!("Campo nao permitido: {key}"),
            ));
        }
    }

    let state = obj.get("state").and_then(|v| v.as_str()).ok_or_else(|| {
        error_response(StatusCode::UNPROCESSABLE_ENTITY, "Campo obrigatorio: state")
    })?;
    let code = STATES
        .iter()
        .find(|(name, _)| *name == state)
        .map(|(_, c)| *c)
        .ok_or_else(|| {
            error_response(
                StatusCode::UNPROCESSABLE_ENTITY,
                &format!("state invalido: {state}"),
            )
        })?;

    let duration = obj
        .get("duration")
        .and_then(|v| v.as_u64())
        .unwrap_or(3000);
    if duration < 50 || duration > 60_000 {
        return Err(error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "duration deve estar entre 50 e 60000",
        ));
    }

    Ok(json!({"type": "expression", "state": code, "duration": duration}))
}

async fn expression(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Json(value): Json<Value>,
) -> Result<Json<Value>, Response> {
    authorized(&state, &headers)?;
    let message = validate_expression(&value)?;
    let bridge = state.bridge.lock().unwrap();
    match bridge.request(message) {
        Ok(ack) => Ok(Json(ack)),
        Err(e) => Err(bridge_error_to_response(e)),
    }
}

/// POST /api/agents/run — executa um agente (codex, claude, local) com um prompt.
/// Retorna o código de saída do processo.
async fn agents_run(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Json(value): Json<Value>,
) -> Result<Json<Value>, Response> {
    authorized(&state, &headers)?;

    let agent = value["agent"].as_str().unwrap_or("");
    let prompt = value["prompt"].as_str().unwrap_or("");
    if agent.is_empty() || prompt.is_empty() {
        return Err(error_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "agent e prompt sao obrigatorios",
        ));
    }

    // Publica eventos na bridge (ESP32).
    let publish = |msg: Value| {
        let _ = state.bridge.lock().unwrap().request(msg);
    };

    let code = match agent {
        "codex" => {
            let args: Vec<String> = vec![prompt.to_string()];
            run_codex(&args, &publish)
        }
        "claude" => {
            let args: Vec<String> = vec![
                "claude".to_string(),
                "-p".to_string(),
                prompt.to_string(),
            ];
            run_command(&args, "Claude Code", &publish)
        }
        "local" => run_ollama("llama3.2", prompt, &publish),
        _ => {
            return Err(error_response(
                StatusCode::UNPROCESSABLE_ENTITY,
                &format!("agent nao suportado: {agent}"),
            ))
        }
    };

    Ok(Json(json!({"ok": code == 0, "exit_code": code})))
}

pub fn create_app(bridge: Arc<Mutex<Bridge>>, key: String) -> Router {
    let state = AppState { bridge, key };
    Router::new()
        .route("/health", get(health))
        .route("/api/status", get(status))
        .route("/api/event", post(event))
        .route("/api/settings", post(settings))
        .route("/api/navigate", post(navigate))
        .route("/api/expression", post(expression))
        .route("/api/agents/run", post(agents_run))
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    struct FakeBridge;

    #[tokio::test]
    async fn health_returns_ok() {
        let app = create_app(
            Arc::new(Mutex::new(Bridge::new("TEST", "expected"))),
            "test-key".into(),
        );
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn auth_required() {
        let app = create_app(
            Arc::new(Mutex::new(Bridge::new("TEST", "expected"))),
            "test-key".into(),
        );
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/status")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}

#[tokio::main]
async fn main() {
    // .tools/ fica na raiz do repo (duas acima de services/host/tokenbuddy-server).
    let manifest = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let config_path = manifest
        .parent() // services/host
        .and_then(|p| p.parent()) // services
        .and_then(|p| p.parent()) // raiz do repo
        .map(|root| root.join(".tools").join("host-config.json"))
        .unwrap_or_else(|| PathBuf::from(".tools/host-config.json"));
    let raw = match std::fs::read_to_string(&config_path) {
        Ok(s) => s.trim_start_matches('\u{feff}').to_string(),
        Err(e) => {
            eprintln!("Config nao encontrada em {} ({e})", config_path.display());
            std::process::exit(1);
        }
    };
    let cfg: Value = serde_json::from_str(&raw).unwrap_or_else(|e| {
        eprintln!("Config invalida: {e}");
        std::process::exit(1);
    });

    let port = cfg["port"].as_str().unwrap_or("COM3").to_string();
    let mac = cfg["mac"].as_str().unwrap_or("").to_string();
    let http_port = cfg["http_port"].as_u64().unwrap_or(8765) as u16;
    let key = cfg["api_key"]
        .as_str()
        .or_else(|| cfg["key"].as_str())
        .unwrap_or("")
        .to_string();

    if key.is_empty() {
        eprintln!("Config: campo 'api_key' obrigatorio");
        std::process::exit(1);
    }

    let bridge = Arc::new(Mutex::new(Bridge::new(&port, &mac)));
    let app = create_app(Arc::clone(&bridge), key);

    // Inicia o worker da bridge com métricas.
    {
        let mut b = bridge.lock().unwrap();
        b.start(|| sample_metrics());
    }

    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], http_port));
    println!("TokenBuddy host em http://127.0.0.1:{http_port}");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
