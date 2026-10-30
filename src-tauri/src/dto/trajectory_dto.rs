use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrajectoryPointDto {
    /// Instant du point (ISO 8601, UTC).
    pub timestamp: String,
    pub latitude_deg: f64,
    pub longitude_deg: f64,
    pub altitude_km: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrajectoryDto {
    pub norad_id: u32,
    /// Horodatage UTC du calcul (ISO 8601).
    pub computed_at: String,
    pub points: Vec<TrajectoryPointDto>,
}
