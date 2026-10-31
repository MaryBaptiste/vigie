use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CometSourceDto {
    Network,
    Cache,
    Offline,
    Loading,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CometCatalogStatusDto {
    /// Date du téléchargement des éléments orbitaux utilisés (ISO 8601).
    pub updated_at: Option<String>,
    pub source: CometSourceDto,
    /// Éléments vieux de plus de 14 jours.
    pub stale: bool,
    pub comet_count: usize,
}
