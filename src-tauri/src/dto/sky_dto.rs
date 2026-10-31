use serde::Serialize;

use crate::dto::{BodyTypeDto, ObserverDto};

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HorizontalDto {
    /// Depuis le nord, vers l'est.
    pub azimuth_deg: f64,
    pub altitude_deg: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum SkyStateDto {
    #[serde(rename = "JOUR")]
    Day,
    #[serde(rename = "CRÉPUSCULE CIVIL")]
    CivilTwilight,
    #[serde(rename = "CRÉPUSCULE NAUTIQUE")]
    NauticalTwilight,
    #[serde(rename = "CRÉPUSCULE ASTRONOMIQUE")]
    AstronomicalTwilight,
    #[serde(rename = "NUIT")]
    Night,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StarDto {
    /// Numéro HR (Bright Star Catalog).
    pub id: u32,
    pub name: Option<String>,
    pub azimuth_deg: f64,
    pub altitude_deg: f64,
    pub magnitude: f64,
    /// Indice de couleur B−V.
    pub color_index: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SegmentDto {
    /// Stable d'un appel à l'autre, pour l'interpolation côté front.
    pub id: u32,
    pub from: HorizontalDto,
    pub to: HorizontalDto,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConstellationDto {
    pub abbreviation: String,
    pub latin_name: String,
    pub french_name: String,
    pub segments: Vec<SegmentDto>,
    /// Barycentre des sommets visibles.
    pub label: Option<HorizontalDto>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkyBodyDto {
    pub id: String,
    pub body_type: BodyTypeDto,
    pub name: String,
    pub azimuth_deg: f64,
    pub altitude_deg: f64,
    pub magnitude: f64,
    /// Lune, Mercure et Vénus : fraction éclairée.
    pub illumination: Option<f64>,
    /// Lune : croissante ou décroissante.
    pub waxing: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkyDto {
    /// Instant du calcul, UTC (ISO 8601).
    pub timestamp: String,
    pub local_sidereal_time_hours: f64,
    pub observer: ObserverDto,
    pub sky_state: SkyStateDto,
    pub sun_altitude_deg: f64,
    /// Étoiles au-dessus de l'horizon (hauteurs apparentes, réfraction incluse).
    pub stars: Vec<StarDto>,
    pub constellations: Vec<ConstellationDto>,
    /// Soleil, Lune, planètes et comètes brillantes, y compris sous l'horizon.
    pub bodies: Vec<SkyBodyDto>,
}
