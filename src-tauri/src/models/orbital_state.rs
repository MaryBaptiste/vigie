use chrono::{DateTime, Utc};

/// Position géodésique WGS84.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GeodeticPosition {
    pub latitude_deg: f64,
    pub longitude_deg: f64,
    pub altitude_km: f64,
}

/// État orbital calculé à un instant donné.
#[derive(Debug, Clone, PartialEq)]
pub struct OrbitalState {
    pub norad_id: u32,
    pub computed_at: DateTime<Utc>,
    pub position: GeodeticPosition,
    pub speed_km_s: f64,
    pub inclination_deg: f64,
    pub period_min: f64,
    pub apogee_km: f64,
    pub perigee_km: f64,
    pub eccentricity: f64,
    pub revolution_number: u64,
    pub tle_epoch: DateTime<Utc>,
    pub tle_age_hours: f64,
}
