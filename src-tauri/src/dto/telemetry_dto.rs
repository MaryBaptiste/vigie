use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TelemetryDto {
    pub norad_id: u32,
    /// Horodatage UTC du calcul (ISO 8601).
    pub timestamp: String,
    pub latitude_deg: f64,
    pub longitude_deg: f64,
    pub altitude_km: f64,
    pub speed_km_s: f64,
    pub inclination_deg: f64,
    pub period_min: f64,
    pub apogee_km: f64,
    pub perigee_km: f64,
    pub eccentricity: f64,
    pub revolution_number: u64,
    /// Époque du TLE (ISO 8601, UTC).
    pub tle_epoch: String,
    pub tle_age_hours: f64,
}
