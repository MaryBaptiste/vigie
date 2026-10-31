use chrono::{DateTime, Utc};

use crate::astronomy::Horizontal;
use crate::models::{CelestialBody, Constellation, Observer, Star};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkyState {
    Day,
    CivilTwilight,
    NauticalTwilight,
    AstronomicalTwilight,
    Night,
}

impl SkyState {
    /// État du ciel d'après la hauteur géométrique du centre du Soleil.
    pub fn from_sun_altitude(altitude_deg: f64) -> Self {
        match altitude_deg {
            h if h > -0.8333 => Self::Day,
            h if h > -6.0 => Self::CivilTwilight,
            h if h > -12.0 => Self::NauticalTwilight,
            h if h > -18.0 => Self::AstronomicalTwilight,
            _ => Self::Night,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VisibleStar<'a> {
    pub star: &'a Star,
    pub horizontal: Horizontal,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VisibleSegment {
    pub id: u32,
    pub from: Horizontal,
    pub to: Horizontal,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VisibleConstellation<'a> {
    pub constellation: &'a Constellation,
    /// Segments dont au moins une extrémité est au-dessus de l'horizon.
    pub segments: Vec<VisibleSegment>,
    /// Barycentre des sommets visibles.
    pub label: Option<Horizontal>,
}

/// État du ciel pour un observateur et un instant.
#[derive(Debug, Clone, PartialEq)]
pub struct SkySnapshot<'a> {
    pub computed_at: DateTime<Utc>,
    pub local_sidereal_deg: f64,
    pub observer: Observer,
    pub sky_state: SkyState,
    pub sun_altitude_deg: f64,
    pub stars: Vec<VisibleStar<'a>>,
    pub constellations: Vec<VisibleConstellation<'a>>,
    pub bodies: Vec<CelestialBody>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_twilight() {
        assert_eq!(SkyState::from_sun_altitude(10.0), SkyState::Day);
        assert_eq!(SkyState::from_sun_altitude(-3.0), SkyState::CivilTwilight);
        assert_eq!(SkyState::from_sun_altitude(-9.0), SkyState::NauticalTwilight);
        assert_eq!(SkyState::from_sun_altitude(-15.0), SkyState::AstronomicalTwilight);
        assert_eq!(SkyState::from_sun_altitude(-30.0), SkyState::Night);
    }
}
