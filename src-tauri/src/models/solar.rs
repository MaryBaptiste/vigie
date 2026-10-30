use chrono::{DateTime, Utc};

/// Point à la surface du globe.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GeoPoint {
    pub latitude_deg: f64,
    pub longitude_deg: f64,
}

/// Point du terminateur (limite jour/nuit).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TerminatorPoint {
    pub point: GeoPoint,
    /// `true` côté matin (lever du soleil), `false` côté soir (coucher).
    pub sunrise: bool,
}

/// Éclairement solaire de la Terre à un instant donné.
#[derive(Debug, Clone, PartialEq)]
pub struct SolarState {
    pub computed_at: DateTime<Utc>,
    pub subsolar: GeoPoint,
    /// Ordonné par longitude croissante, de −180° à +180°.
    pub terminator: Vec<TerminatorPoint>,
}
