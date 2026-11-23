//! TokenBuddy core: bridge USB serial, métricas de host e conectores de agente.
//!
//! Este crate não depende de UI nem de HTTP; o servidor (`tokenbuddy-server`)
//! e a CLI (`tokenbuddy-cli`) consomem apenas esta API pública.

pub mod bridge;
pub mod connectors;
pub mod metrics;
pub mod state;
