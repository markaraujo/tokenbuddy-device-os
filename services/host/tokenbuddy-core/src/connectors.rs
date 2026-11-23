//! Conectores de agente: eventos oficiais do Codex (JSONL) e comandos genéricos.
//!
//! Regra do projeto: nunca inferir uso a partir de texto de terminal — apenas
//! os eventos `turn.*` emitidos pelo `codex exec --json` viram eventos de agente.

use serde_json::{json, Value};
use std::process::{Command, Stdio};

/// Converte um evento JSONL do Codex em evento de agente, se aplicável.
///
/// Retorna `None` para qualquer tipo que não seja `turn.started`, `turn.failed`
/// ou `turn.completed`.
pub fn codex_event(value: &Value) -> Option<Value> {
    let kind = value.get("type")?.as_str()?;
    match kind {
        "turn.started" => Some(json!({"event": "agent.started", "agent": "Codex"})),
        "turn.failed" => Some(json!({"event": "agent.failed", "agent": "Codex"})),
        "turn.completed" => {
            let mut result = json!({"event": "agent.completed", "agent": "Codex"});
            if let Some(usage) = value.get("usage").filter(|u| !u.is_null()) {
                let input = usage.get("input_tokens").and_then(|v| v.as_u64());
                let output = usage.get("output_tokens").and_then(|v| v.as_u64());
                // Só publica tokens quando ambos são inteiros no domínio i32.
                if let (Some(i), Some(o)) = (input, output) {
                    if i <= 2_147_483_647 && o <= 2_147_483_647 {
                        result["input_tokens"] = json!(i);
                        result["output_tokens"] = json!(o);
                    }
                }
            }
            Some(result)
        }
        _ => None,
    }
}

/// Executa `codex exec --json <arguments>` e publica eventos via `publish`.
///
/// Retorna o código de saída do processo. Se o processo terminar com erro e
/// nenhum evento terminal tenha sido publicado, publica `agent.failed`.
pub fn run_codex(arguments: &[String], publish: &dyn Fn(Value)) -> i32 {
    let task = uuid::Uuid::new_v4().simple().to_string();
    let mut child = Command::new("codex")
        .arg("exec")
        .arg("--json")
        .args(arguments)
        .stdout(Stdio::piped())
        .spawn()
        .expect("falha ao iniciar codex");

    let stdout = child.stdout.take().unwrap();
    let mut reader = std::io::BufReader::new(stdout);
    let mut published_terminal = false;
    loop {
        use std::io::BufRead;
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => {
                print!("{line}");
                if let Ok(value) = serde_json::from_str::<Value>(&line) {
                    if let Some(event) = codex_event(&value) {
                        let is_terminal = event["event"].as_str() != Some("agent.started");
                        publish(json!({
                            "event": event["event"],
                            "agent": "Codex",
                            "task_id": task,
                            "input_tokens": event.get("input_tokens"),
                            "output_tokens": event.get("output_tokens"),
                        }));
                        published_terminal = is_terminal;
                    }
                }
            }
            Err(_) => break,
        }
    }
    let code = child.wait().expect("falha ao aguardar codex").code().unwrap_or(1);
    if code != 0 && !published_terminal {
        publish(json!({"event": "agent.failed", "agent": "Codex", "task_id": task}));
    }
    code
}

/// Executa um comando genérico como agente, publicando started/completed|failed.
///
/// Retorna o código de saída (ou -1 se o processo não pôde ser iniciado).
pub fn run_command(arguments: &[String], agent: &str, publish: &dyn Fn(Value)) -> i32 {
    let task = uuid::Uuid::new_v4().simple().to_string();
    publish(json!({"event": "agent.started", "agent": agent, "task_id": task}));
    let code = Command::new(&arguments[0])
        .args(&arguments[1..])
        .status()
        .map(|s| s.code().unwrap_or(-1))
        .unwrap_or(-1);
    publish(json!({
        "event": if code == 0 { "agent.completed" } else { "agent.failed" },
        "agent": agent,
        "task_id": task,
    }));
    code
}

/// Executa `ollama run <model> <prompt>` e lê o uso de tokens da API local.
///
/// O processo CLI não expõe contagem de tokens; o host consulta
/// `GET http://127.0.0.1:11434/api/ps` antes e depois para atribuir o delta
/// ao modelo em execução. Se a API não estiver acessível, publica sem tokens.
pub fn run_ollama(model: &str, prompt: &str, publish: &dyn Fn(Value)) -> i32 {
    let task = uuid::Uuid::new_v4().simple().to_string();
    publish(json!({"event": "agent.started", "agent": "IA Local", "task_id": task}));

    // Linha de base antes do run (tokens acumulados no modelo, se já carregado).
    let baseline = fetch_ollama_ps()
        .and_then(|ps| model_tokens(&ps, model))
        .unwrap_or(0);

    let args: Vec<String> = vec![
        "ollama".to_string(),
        "run".to_string(),
        model.to_string(),
        prompt.to_string(),
    ];
    let code = Command::new(&args[0])
        .args(&args[1..])
        .status()
        .map(|s| s.code().unwrap_or(-1))
        .unwrap_or(-1);

    // Estado final: delta = final - baseline (nunca negativo).
    let final_tokens = fetch_ollama_ps()
        .and_then(|ps| model_tokens(&ps, model));
    let input_tokens = final_tokens.map(|f| f.saturating_sub(baseline));

    publish(json!({
        "event": if code == 0 { "agent.completed" } else { "agent.failed" },
        "agent": "IA Local",
        "task_id": task,
        "input_tokens": input_tokens,
        "output_tokens": input_tokens.and(Some(0)), // placeholder: /api/ps não separa output
    }));
    code
}

/// GET http://127.0.0.1:11434/api/ps — retorna o JSON bruto ou None.
fn fetch_ollama_ps() -> Option<Value> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(2))
        .build()
        .ok()?;
    let resp = client
        .get("http://127.0.0.1:11434/api/ps")
        .send()
        .ok()?;
    if !resp.status().is_success() {
        return None;
    }
    resp.json::<Value>().ok()
}

/// Extrai `tokens` do modelo listado em `/api/ps`.
fn model_tokens(ps: &Value, model: &str) -> Option<u64> {
    let models = ps.get("models")?.as_array()?;
    for m in models {
        if m.get("name").and_then(|v| v.as_str()) == Some(model) {
            return m.get("tokens").and_then(|v| v.as_u64());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codex_official_events() {
        assert_eq!(codex_event(&json!({"type": "turn.started"})).unwrap()["event"], "agent.started");
        assert_eq!(codex_event(&json!({"type": "turn.failed"})).unwrap()["event"], "agent.failed");
        assert!(codex_event(&json!({"type": "item.completed"})).is_none());
        assert!(codex_event(&json!({"type": "error"})).is_none());

        let event = codex_event(&json!({
            "type": "turn.completed",
            "usage": {"input_tokens": 10, "cached_input_tokens": 8, "output_tokens": 2}
        }))
        .unwrap();
        assert_eq!(event["input_tokens"], 10);
        assert_eq!(event["output_tokens"], 2);
        assert!(event.get("context").is_none());

        // Sem usage → sem tokens (nunca inventar).
        let event = codex_event(&json!({"type": "turn.completed"})).unwrap();
        assert!(event.get("input_tokens").is_none());

        // Token booleano não é int → ignorado.
        let event = codex_event(
            &json!({"type": "turn.completed", "usage": {"input_tokens": true, "output_tokens": 2}}),
        )
        .unwrap();
        assert!(event.get("input_tokens").is_none());

        // Token fora do domínio i32 → ignorado.
        let event = codex_event(
            &json!({"type": "turn.completed", "usage": {"input_tokens": 5_000_000_000u64, "output_tokens": 2}}),
        )
        .unwrap();
        assert!(event.get("input_tokens").is_none());
    }

    #[test]
    fn run_command_success_and_failure() {
        for code in [0i32, 7] {
            let events = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
            let ev = events.clone();
            let args: Vec<String> = if cfg!(windows) {
                vec![
                    "powershell".to_string(),
                    "-NoProfile".to_string(),
                    "-Command".to_string(),
                    format!("exit {}", code),
                ]
            } else {
                vec!["sh".to_string(), "-c".to_string(), format!("exit {}", code)]
            };
            let code_result = run_command(&args, "Test", &|v: Value| ev.lock().unwrap().push(v));
            assert_eq!(code_result, code);
            let events = events.lock().unwrap();
            assert_eq!(events[0]["event"], "agent.started");
            assert_eq!(
                events[1]["event"],
                if code == 0 { "agent.completed" } else { "agent.failed" }
            );
            assert_eq!(events[0]["task_id"], events[1]["task_id"]);
        }
    }
}
