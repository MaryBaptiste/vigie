//! Positions des objets du système solaire vues par un observateur à un
//! instant : enchaîne les calculs de `astronomy/` et construit les modèles.

use crate::astronomy::comet::{comet_magnitude, comet_position};
use crate::astronomy::coordinates::{angular_separation_deg, equatorial_to_horizontal, refraction_deg, topocentric};
use crate::astronomy::linalg::Vec3;
use crate::astronomy::moon::{moon_magnitude, moon_phase, moon_position, moon_semidiameter_deg};
use crate::astronomy::planets::planet_position;
use crate::astronomy::rise_set::{STAR_H0_DEG, SUN_H0_DEG, moon_h0_deg};
use crate::astronomy::sun::{SunPosition, sun_position};
use crate::astronomy::{Epoch, Equatorial, GeoSite, Horizontal, Planet};
use crate::models::{BodyKind, CelestialBody, Comet, MoonPhaseKind, Visibility};

/// Magnitude limite à l'œil nu.
pub const NAKED_EYE_LIMIT: f64 = 6.0;
/// Magnitude limite des comètes retenues (jumelles, petit instrument).
pub const COMET_LIMIT: f64 = 12.0;
/// En deçà, l'objet est perdu dans l'éclat du Soleil.
const MIN_ELONGATION_DEG: f64 = 10.0;

/// Objet dont on veut la position.
#[derive(Debug, Clone, Copy)]
pub enum Target<'a> {
    Sun,
    Moon,
    Planet(Planet),
    Comet(&'a Comet),
}

impl Target<'_> {
    /// Hauteur de référence pour le lever et le coucher.
    pub fn horizon_altitude_deg(&self, body: &CelestialBody) -> f64 {
        match self {
            Target::Sun => SUN_H0_DEG,
            Target::Moon => moon_h0_deg(moon_semidiameter_deg(body.distance_au * crate::astronomy::moon::KM_PER_AU)),
            _ => STAR_H0_DEG,
        }
    }
}

/// Un instant et un lieu : tout ce qui est commun aux objets calculés.
pub struct Situation {
    pub epoch: Epoch,
    pub site: GeoSite,
    pub local_sidereal_deg: f64,
    sun: SunPosition,
}

impl Situation {
    pub fn new(jd: f64, site: GeoSite) -> Self {
        let epoch = Epoch::from_julian_day(jd);
        Self {
            local_sidereal_deg: epoch.local_sidereal_deg(site.longitude_deg),
            sun: sun_position(&epoch),
            epoch,
            site,
        }
    }

    pub fn geometric_horizontal(&self, position: Equatorial) -> Horizontal {
        equatorial_to_horizontal(position, self.local_sidereal_deg, self.site.latitude_deg)
    }

    /// Position apparente (réfraction incluse) d'une direction de l'équateur moyen J2000.
    pub fn apparent_horizontal_of_j2000(&self, v: Vec3) -> Horizontal {
        refracted(self.geometric_horizontal(self.epoch.apparent_from_j2000(v)))
    }

    /// Hauteur géométrique topocentrique du centre du Soleil.
    pub fn sun_altitude_deg(&self) -> f64 {
        let topo = topocentric(self.sun.equatorial, self.sun.distance_au, self.local_sidereal_deg, &self.site);
        self.geometric_horizontal(topo).altitude_deg
    }

    pub fn comet_magnitude(&self, comet: &Comet) -> f64 {
        let position = comet_position(&comet.elements, &self.epoch);
        comet_magnitude(
            comet.absolute_magnitude,
            comet.slope_k,
            position.distance_au,
            position.heliocentric_distance_au,
        )
    }

    pub fn body(&self, target: &Target) -> CelestialBody {
        let mut phase_angle_deg = None;
        let mut illumination = None;
        let mut waxing = None;
        let mut moon_phase_kind = None;
        let (id, name, kind, geocentric, distance_au, heliocentric_distance_au, magnitude) = match *target {
            Target::Sun => (
                "sun".to_string(),
                "Soleil".to_string(),
                BodyKind::Sun,
                self.sun.equatorial,
                self.sun.distance_au,
                None,
                -26.74,
            ),
            Target::Moon => {
                let moon = moon_position(&self.epoch);
                let phase = moon_phase(&moon, &self.sun);
                phase_angle_deg = Some(phase.phase_angle_deg);
                illumination = Some(phase.illumination);
                waxing = Some(phase.waxing);
                moon_phase_kind = Some(MoonPhaseKind::from_elongation(phase.elongation_deg));
                (
                    "moon".to_string(),
                    "Lune".to_string(),
                    BodyKind::Moon,
                    moon.equatorial,
                    moon.distance_au(),
                    None,
                    moon_magnitude(phase.phase_angle_deg),
                )
            }
            Target::Planet(planet) => {
                let position = planet_position(planet, &self.epoch);
                if matches!(planet, Planet::Mercury | Planet::Venus) {
                    phase_angle_deg = Some(position.phase_angle_deg);
                    illumination = Some(position.illumination);
                }
                (
                    BodyKind::planet_id(planet).to_string(),
                    BodyKind::planet_name(planet).to_string(),
                    BodyKind::Planet(planet),
                    position.equatorial,
                    position.distance_au,
                    Some(position.heliocentric_distance_au),
                    position.magnitude,
                )
            }
            Target::Comet(comet) => {
                let position = comet_position(&comet.elements, &self.epoch);
                (
                    format!("comet:{}", comet.designation),
                    comet.name.clone(),
                    BodyKind::Comet,
                    position.equatorial,
                    position.distance_au,
                    Some(position.heliocentric_distance_au),
                    comet_magnitude(
                        comet.absolute_magnitude,
                        comet.slope_k,
                        position.distance_au,
                        position.heliocentric_distance_au,
                    ),
                )
            }
        };

        let equatorial = topocentric(geocentric, distance_au, self.local_sidereal_deg, &self.site);
        let geometric = self.geometric_horizontal(equatorial);
        CelestialBody {
            id,
            name,
            kind,
            equatorial,
            horizontal: refracted(geometric),
            geometric_altitude_deg: geometric.altitude_deg,
            magnitude,
            distance_au,
            heliocentric_distance_au,
            elongation_deg: angular_separation_deg(geocentric, self.sun.equatorial),
            phase_angle_deg,
            illumination,
            waxing,
            moon_phase: moon_phase_kind,
        }
    }
}

/// Ajoute la réfraction atmosphérique à une position géométrique.
pub fn refracted(geometric: Horizontal) -> Horizontal {
    Horizontal {
        azimuth_deg: geometric.azimuth_deg,
        altitude_deg: geometric.altitude_deg + refraction_deg(geometric.altitude_deg),
    }
}

/// Visibilité à l'œil nu. La Lune reste visible de jour, sauf trop près du Soleil.
pub fn visibility(body: &CelestialBody, sun_altitude_deg: f64) -> Visibility {
    let sun_up = sun_altitude_deg > SUN_H0_DEG && body.kind != BodyKind::Moon;
    if body.horizontal.altitude_deg < 0.0 {
        Visibility::BelowHorizon
    } else if sun_up || body.elongation_deg < MIN_ELONGATION_DEG {
        Visibility::LostInDaylight
    } else if body.magnitude > NAKED_EYE_LIMIT {
        Visibility::TooFaint
    } else {
        Visibility::Visible
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn paris() -> GeoSite {
        GeoSite { latitude_deg: 48.8566, longitude_deg: 2.3522, altitude_m: 35.0 }
    }

    #[test]
    fn topocentric_positions_match_jpl_horizons() {
        // Horizons, Paris, 2026-10-30 20:00 UT, az/el apparents sans réfraction.
        // La Lune, à 1.2° de hauteur, valide la correction de parallaxe (~1°).
        let situation = Situation::new(2_461_344.333_333_333, paris());
        let cases = [
            (Target::Sun, 289.412_696, -34.142_495),
            (Target::Moon, 50.357_218, 1.199_600),
            (Target::Planet(Planet::Mars), 21.167_924, -21.732_871),
            (Target::Planet(Planet::Jupiter), 15.934_791, -25.685_461),
            (Target::Planet(Planet::Saturn), 144.011_511, 36.599_836),
        ];
        for (target, az, alt) in cases {
            let body = situation.body(&target);
            let alt_error = (body.geometric_altitude_deg - alt).abs();
            let az_error = (body.horizontal.azimuth_deg - az).abs() * alt.to_radians().cos();
            let tolerance = if matches!(target, Target::Moon) { 0.02 } else { 0.01 };
            assert!(alt_error < tolerance && az_error < tolerance, "{}: {alt_error} {az_error}", body.name);
        }
    }

    #[test]
    fn classifies_visibility() {
        let situation = Situation::new(2_461_344.333_333_333, paris());
        let saturn = situation.body(&Target::Planet(Planet::Saturn));
        assert_eq!(visibility(&saturn, -34.0), Visibility::Visible);
        assert_eq!(visibility(&saturn, 10.0), Visibility::LostInDaylight);
        let neptune = situation.body(&Target::Planet(Planet::Neptune));
        assert_eq!(visibility(&neptune, -34.0), Visibility::TooFaint);
        let mars = situation.body(&Target::Planet(Planet::Mars));
        assert_eq!(visibility(&mars, -34.0), Visibility::BelowHorizon);
        let moon = situation.body(&Target::Moon);
        assert_eq!(visibility(&moon, 10.0), Visibility::Visible);
    }

    #[test]
    fn sun_altitude_drives_sky_state() {
        assert!((Situation::new(2_461_344.333_333_333, paris()).sun_altitude_deg() + 34.14).abs() < 0.01);
    }
}
