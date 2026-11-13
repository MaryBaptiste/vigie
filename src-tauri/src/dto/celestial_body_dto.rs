use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BodyTypeDto {
    Sun,
    Moon,
    Planet,
    Comet,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum VisibilityDto {
    #[serde(rename = "VISIBLE")]
    Visible,
    #[serde(rename = "BELOW HORIZON")]
    BelowHorizon,
    #[serde(rename = "LOST IN DAYLIGHT")]
    LostInDaylight,
    #[serde(rename = "TOO FAINT")]
    TooFaint,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConstellationRefDto {
    pub abbreviation: String,
    pub latin_name: String,
    pub french_name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NightCurveDto {
    /// Début et fin de la courbe (coucher et lever du Soleil hors régions polaires).
    pub start: String,
    pub end: String,
    pub sunset: Option<String>,
    pub sunrise: Option<String>,
    /// Hauteurs géométriques régulièrement espacées de `start` à `end`.
    pub altitudes: Vec<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CelestialBodyDto {
    pub id: String,
    pub body_type: BodyTypeDto,
    pub name: String,
    pub status: VisibilityDto,
    pub azimuth_deg: f64,
    pub altitude_deg: f64,
    pub right_ascension_deg: f64,
    pub declination_deg: f64,
    pub magnitude: f64,
    pub distance_au: f64,
    /// Lune seulement.
    pub distance_km: Option<f64>,
    pub heliocentric_distance_au: Option<f64>,
    pub elongation_deg: f64,
    pub constellation: Option<ConstellationRefDto>,
    /// Événements de la nuit en cours, UTC (ISO 8601).
    pub rise: Option<String>,
    pub transit: Option<String>,
    pub set: Option<String>,
    pub transit_altitude_deg: Option<f64>,
    pub max_night_altitude_deg: Option<f64>,
    /// Lune, Mercure et Vénus.
    pub illumination: Option<f64>,
    pub phase_angle_deg: Option<f64>,
    /// Lune seulement.
    pub waxing: Option<bool>,
    pub phase_name: Option<String>,
    pub night: NightCurveDto,
}
