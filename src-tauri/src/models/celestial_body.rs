use chrono::{DateTime, Utc};

use crate::astronomy::{Equatorial, Horizontal, Planet};
use crate::models::Constellation;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BodyKind {
    Sun,
    Moon,
    Planet(Planet),
    Comet,
}

impl BodyKind {
    pub fn planet_id(planet: Planet) -> &'static str {
        match planet {
            Planet::Mercury => "mercury",
            Planet::Venus => "venus",
            Planet::Mars => "mars",
            Planet::Jupiter => "jupiter",
            Planet::Saturn => "saturn",
            Planet::Uranus => "uranus",
            Planet::Neptune => "neptune",
        }
    }

    pub fn planet_name(planet: Planet) -> &'static str {
        match planet {
            Planet::Mercury => "Mercury",
            Planet::Venus => "Venus",
            Planet::Mars => "Mars",
            Planet::Jupiter => "Jupiter",
            Planet::Saturn => "Saturn",
            Planet::Uranus => "Uranus",
            Planet::Neptune => "Neptune",
        }
    }
}

/// Position et photométrie d'un objet du système solaire à un instant donné,
/// vu depuis l'observateur.
#[derive(Debug, Clone, PartialEq)]
pub struct CelestialBody {
    pub id: String,
    pub name: String,
    pub kind: BodyKind,
    /// Coordonnées apparentes topocentriques (équateur vrai de la date).
    pub equatorial: Equatorial,
    /// Position apparente, réfraction incluse.
    pub horizontal: Horizontal,
    /// Hauteur géométrique (sans réfraction), utilisée pour les levers et couchers.
    pub geometric_altitude_deg: f64,
    pub magnitude: f64,
    pub distance_au: f64,
    pub heliocentric_distance_au: Option<f64>,
    /// Élongation géocentrique par rapport au Soleil, en degrés.
    pub elongation_deg: f64,
    pub phase_angle_deg: Option<f64>,
    pub illumination: Option<f64>,
    /// Lune seulement : croissante ou décroissante, et nom de la phase.
    pub waxing: Option<bool>,
    pub moon_phase: Option<MoonPhaseKind>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoonPhaseKind {
    New,
    WaxingCrescent,
    FirstQuarter,
    WaxingGibbous,
    Full,
    WaningGibbous,
    LastQuarter,
    WaningCrescent,
}

impl MoonPhaseKind {
    /// Phase nommée d'après l'élongation λ☾ − λ☉ (0° = nouvelle lune).
    pub fn from_elongation(elongation_deg: f64) -> Self {
        match elongation_deg {
            e if !(7.5..352.5).contains(&e) => Self::New,
            e if e < 82.5 => Self::WaxingCrescent,
            e if e < 97.5 => Self::FirstQuarter,
            e if e < 172.5 => Self::WaxingGibbous,
            e if e < 187.5 => Self::Full,
            e if e < 262.5 => Self::WaningGibbous,
            e if e < 277.5 => Self::LastQuarter,
            _ => Self::WaningCrescent,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
    Visible,
    BelowHorizon,
    /// Soleil levé ou objet trop proche du Soleil.
    LostInDaylight,
    /// Magnitude > 6 : nécessite un instrument.
    TooFaint,
}

/// Nuit de référence et pas d'échantillonnage de la courbe de hauteur.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NightInfo {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub sunset: Option<DateTime<Utc>>,
    pub sunrise: Option<DateTime<Utc>>,
}

/// Fiche complète d'un objet pour la page « Objets visibles ».
#[derive(Debug, Clone, PartialEq)]
pub struct BodyReport<'a> {
    pub body: CelestialBody,
    pub visibility: Visibility,
    pub constellation: Option<&'a Constellation>,
    pub rise: Option<DateTime<Utc>>,
    pub transit: Option<DateTime<Utc>>,
    pub transit_altitude_deg: Option<f64>,
    pub set: Option<DateTime<Utc>>,
    pub max_night_altitude_deg: Option<f64>,
    /// Hauteurs régulièrement espacées du début à la fin de la nuit.
    pub night_altitudes: Vec<f64>,
    pub night: NightInfo,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_moon_phases() {
        assert_eq!(MoonPhaseKind::from_elongation(2.0), MoonPhaseKind::New);
        assert_eq!(MoonPhaseKind::from_elongation(355.0), MoonPhaseKind::New);
        assert_eq!(MoonPhaseKind::from_elongation(90.0), MoonPhaseKind::FirstQuarter);
        assert_eq!(MoonPhaseKind::from_elongation(180.0), MoonPhaseKind::Full);
        assert_eq!(MoonPhaseKind::from_elongation(300.0), MoonPhaseKind::WaningCrescent);
    }
}
