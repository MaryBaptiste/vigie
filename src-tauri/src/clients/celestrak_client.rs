use std::time::Duration;

use crate::error::{AppError, AppResult};

const BASE_URL: &str = "https://celestrak.org/NORAD/elements/gp.php";
const USER_AGENT: &str = concat!("Vigie/", env!("CARGO_PKG_VERSION"));

/// Accès HTTP brut à CelesTrak (format GP / OMM JSON).
pub struct CelestrakClient {
    http: reqwest::Client,
}

impl CelestrakClient {
    pub fn new() -> AppResult<Self> {
        let http = reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .timeout(Duration::from_secs(10))
            .build()?;
        Ok(Self { http })
    }

    /// Récupère les éléments GP d'un objet. Renvoie `None` si CelesTrak ne
    /// connaît pas ce numéro NORAD.
    pub async fn fetch_gp(&self, norad_id: u32) -> AppResult<Option<sgp4::Elements>> {
        let body = self
            .http
            .get(BASE_URL)
            .query(&[("CATNR", norad_id.to_string()), ("FORMAT", "json".into())])
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;

        // Pour un identifiant inconnu, CelesTrak répond en texte brut
        // ("No GP data found") et non en JSON.
        if body.trim_start().starts_with("No GP data") {
            return Ok(None);
        }

        let mut records: Vec<sgp4::Elements> =
            serde_json::from_str(&body).map_err(|e| AppError::InvalidResponse(e.to_string()))?;
        Ok(if records.is_empty() {
            None
        } else {
            Some(records.swap_remove(0))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Vérifie sur CelesTrak que le catalogue en dur est valide.
    /// Nécessite le réseau : `cargo test -- --ignored`.
    #[tokio::test]
    #[ignore]
    async fn catalogue_ids_exist_on_celestrak() {
        let client = CelestrakClient::new().unwrap();
        for object in crate::facades::space_object_facade::CATALOGUE {
            let elements = client.fetch_gp(object.norad_id).await.unwrap();
            let elements = elements.unwrap_or_else(|| panic!("{} not found", object.name));
            assert_eq!(elements.norad_id, u64::from(object.norad_id));
        }
    }
}
