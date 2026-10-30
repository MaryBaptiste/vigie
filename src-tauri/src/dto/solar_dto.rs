use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminatorPointDto {
    pub latitude_deg: f64,
    pub longitude_deg: f64,
    /// `true` côté lever du soleil, `false` côté coucher.
    pub sunrise: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SolarDto {
    /// Horodatage UTC du calcul (ISO 8601).
    pub timestamp: String,
    pub subsolar_latitude_deg: f64,
    pub subsolar_longitude_deg: f64,
    /// Ordonné par longitude croissante, de −180° à +180°.
    pub terminator: Vec<TerminatorPointDto>,
}
