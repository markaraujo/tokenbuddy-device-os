//! Derivação mínima de estado a partir do snapshot da bridge.
//!
//! Uma única função, um match simples — o "porquê" é devolvido junto para a
//! UI mostrar ("feedback na rarinha"). Sem histerese, sem cooldown.

use serde_json::Value;

/// Estados que o firmware conhece (mesma ordem de `behavior.h`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Disconnected,
    Error,
    Working,
    Success,
    Idle,
}

impl State {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Disconnected => "disconnected",
            Self::Error => "error",
            Self::Working => "working",
            Self::Success => "success",
            Self::Idle => "idle",
        }
    }
}

/// Deriva o estado do avatar a partir do snapshot da bridge.
///
/// Prioridade: desconexão > erro recente > tarefa ativa (started sem terminal)
/// > último evento terminal > ocioso. O motivo é uma frase curta para a UI.
pub fn next_state(snapshot: &Value) -> (State, &'static str) {
    let connected = snapshot.get("serial_connected").and_then(|v| v.as_bool()) == Some(true);
    if !connected {
        return (State::Disconnected, "USB nao conectado");
    }

    // Erro recente: última falha do dispositivo ou erro de transporte.
    if let Some(err) = snapshot.get("last_error").and_then(|v| v.as_str()) {
        if !err.is_empty() {
            return (State::Error, "Dispositivo reportou erro");
        }
    }

    // Tarefa de agente ativa: started sem evento terminal correspondente.
    if let Some(agent) = snapshot.get("last_agent_event") {
        let event = agent.get("event").and_then(|v| v.as_str()).unwrap_or("");
        match event {
            "agent.started" => return (State::Working, "Agente trabalhando"),
            "agent.completed" => return (State::Success, "Tarefa concluida"),
            "agent.failed" => return (State::Error, "Tarefa falhou"),
            _ => {}
        }
    }

    (State::Idle, "Aguardando atividade")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn disconnected_wins() {
        let snap = json!({"serial_connected": false, "last_error": "x", "last_agent_event": {"event": "agent.started"}});
        assert_eq!(next_state(&snap).0, State::Disconnected);
    }

    #[test]
    fn error_beats_agent() {
        let snap = json!({"serial_connected": true, "last_error": "usb gone", "last_agent_event": {"event": "agent.started"}});
        assert_eq!(next_state(&snap).0, State::Error);
    }

    #[test]
    fn active_task_is_working() {
        let snap = json!({"serial_connected": true, "last_error": null, "last_agent_event": {"event": "agent.started"}});
        assert_eq!(next_state(&snap).0, State::Working);
    }

    #[test]
    fn terminal_events() {
        let ok = json!({"serial_connected": true, "last_error": null, "last_agent_event": {"event": "agent.completed"}});
        assert_eq!(next_state(&ok).0, State::Success);
        let bad = json!({"serial_connected": true, "last_error": null, "last_agent_event": {"event": "agent.failed"}});
        assert_eq!(next_state(&bad).0, State::Error);
    }

    #[test]
    fn idle_when_nothing() {
        let snap = json!({"serial_connected": true, "last_error": null});
        assert_eq!(next_state(&snap).0, State::Idle);
    }
}
