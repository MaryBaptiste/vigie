use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ObserverDto {
    pub latitude_deg: f64,
    /// Positive vers l'est.
    pub longitude_deg: f64,
    pub altitude_m: f64,
    pub label: String,
}
