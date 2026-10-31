use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};

/// Erreur commune à toutes les couches, sérialisée vers le front sous la forme
/// `{ kind: "network" | ..., message: "..." }`.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("network error: {0}")]
    Network(#[from] reqwest::Error),

    /// Réponse illisible d'un service distant (CelesTrak, Minor Planet Center).
    #[error("invalid upstream response: {0}")]
    InvalidResponse(String),

    #[error("no orbital data found for NORAD {0}")]
    NotFound(u32),

    #[error("orbit propagation failed: {0}")]
    Propagation(String),

    #[error("upstream unavailable, retry in {retry_in_secs}s: {message}")]
    Backoff { retry_in_secs: u64, message: String },

    #[error("invalid input: {0}")]
    InvalidInput(String),

    #[error("storage error: {0}")]
    Storage(String),

    #[error("embedded catalog is corrupted: {0}")]
    Catalog(String),
}

impl AppError {
    pub fn kind(&self) -> &'static str {
        match self {
            AppError::Network(_) | AppError::Backoff { .. } => "network",
            AppError::InvalidResponse(_) => "invalidResponse",
            AppError::NotFound(_) => "notFound",
            AppError::Propagation(_) => "propagation",
            AppError::InvalidInput(_) => "invalidInput",
            AppError::Storage(_) => "storage",
            AppError::Catalog(_) => "catalog",
        }
    }
}

impl Serialize for AppError {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("AppError", 2)?;
        state.serialize_field("kind", self.kind())?;
        state.serialize_field("message", &self.to_string())?;
        state.end()
    }
}

pub type AppResult<T> = Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_kind_and_message() {
        let json = serde_json::to_value(AppError::NotFound(42)).unwrap();
        assert_eq!(json["kind"], "notFound");
        assert_eq!(json["message"], "no orbital data found for NORAD 42");

        let json = serde_json::to_value(AppError::InvalidInput("latitude out of range".into())).unwrap();
        assert_eq!(json["kind"], "invalidInput");
        assert_eq!(json["message"], "invalid input: latitude out of range");
    }
}
