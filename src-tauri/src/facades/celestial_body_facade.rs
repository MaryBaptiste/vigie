use std::sync::Arc;

use chrono::Utc;

use crate::astronomy::Planet;
use crate::astronomy::rise_set::{
    AltitudeSample, SUN_H0_DEG, altitude_at, max_altitude_between, night_window, rise_transit_set, sample_altitudes,
};
use crate::astronomy::time::{datetime_from_julian_day, julian_day};
use crate::catalogs::Catalogs;
use crate::dto::{CelestialBodyDto, CometCatalogStatusDto};
use crate::error::AppResult;
use crate::facades::ephemeris::{COMET_LIMIT, Situation, Target, visibility};
use crate::facades::{CometCatalog, ObserverFacade, parse_instant};
use crate::mappers::{CelestialBodyMapper, CometCatalogMapper};
use crate::models::{BodyReport, NightInfo};

/// Pas d'échantillonnage des courbes de hauteur : 10 minutes.
const SAMPLE_STEP_DAYS: f64 = 10.0 / 1440.0;
/// Nombre de points de la courbe de hauteur nocturne envoyée au front.
const NIGHT_CURVE_POINTS: usize = 48;

pub struct CelestialBodyFacade {
    catalogs: Arc<Catalogs>,
    observer: ObserverFacade,
    comets: Arc<CometCatalog>,
}

impl CelestialBodyFacade {
    pub fn new(catalogs: Arc<Catalogs>, observer: ObserverFacade, comets: Arc<CometCatalog>) -> Self {
        Self { catalogs, observer, comets }
    }

    /// Lune, planètes et comètes de magnitude ≤ 12, avec leurs événements de la nuit.
    pub fn get_visible_bodies(&self, datetime_utc: Option<&str>) -> AppResult<Vec<CelestialBodyDto>> {
        let instant = parse_instant(datetime_utc)?;
        let site = self.observer.current().site();
        let jd = julian_day(&instant);
        let now = Situation::new(jd, site);

        // Nuit de référence, puis fenêtre de 24 h centrée sur elle pour les événements.
        let sun = sample_altitudes(jd - 1.0, jd + 1.5, SAMPLE_STEP_DAYS, |t| Situation::new(t, site).sun_altitude_deg());
        let window = night_window(&sun, jd);
        let middle = window.middle();
        let situations: Vec<Situation> = sample_altitudes(middle - 0.5, middle + 0.5, SAMPLE_STEP_DAYS, |_| 0.0)
            .iter()
            .map(|s| Situation::new(s.jd, site))
            .collect();
        let sun_ever_sets = sun.iter().any(|s| s.altitude_deg < SUN_H0_DEG);

        let night = NightInfo {
            start: datetime_from_julian_day(window.start),
            end: datetime_from_julian_day(window.end),
            sunset: window.sunset.map(datetime_from_julian_day),
            sunrise: window.sunrise.map(datetime_from_julian_day),
        };

        let (comets, _) = self.comets.snapshot();
        let targets = [Target::Moon]
            .into_iter()
            .chain(Planet::ALL.map(Target::Planet))
            .chain(comets.iter().filter(|c| now.comet_magnitude(c) <= COMET_LIMIT).map(Target::Comet));

        let sun_altitude = now.sun_altitude_deg();
        let reports: Vec<BodyReport> = targets
            .map(|target| {
                let body = now.body(&target);
                let samples: Vec<AltitudeSample> = situations
                    .iter()
                    .map(|s| AltitudeSample { jd: s.epoch.jd, altitude_deg: s.body(&target).geometric_altitude_deg })
                    .collect();
                let events = rise_transit_set(&samples, target.horizon_altitude_deg(&body));
                let step = (window.end - window.start) / (NIGHT_CURVE_POINTS - 1) as f64;
                let night_altitudes = (0..NIGHT_CURVE_POINTS)
                    .filter_map(|i| altitude_at(&samples, window.start + i as f64 * step))
                    .collect();
                BodyReport {
                    visibility: visibility(&body, sun_altitude),
                    constellation: self.catalogs.constellation_at(body.equatorial, now.epoch.jde),
                    rise: events.rise.map(datetime_from_julian_day),
                    transit: events.transit.map(datetime_from_julian_day),
                    transit_altitude_deg: events.transit_altitude_deg,
                    set: events.set.map(datetime_from_julian_day),
                    max_night_altitude_deg: sun_ever_sets
                        .then(|| max_altitude_between(&samples, window.start, window.end))
                        .flatten(),
                    night_altitudes,
                    night,
                    body,
                }
            })
            .collect();

        Ok(reports.iter().map(CelestialBodyMapper::to_dto).collect())
    }

    pub fn get_comet_catalog_status(&self) -> CometCatalogStatusDto {
        let (_, status) = self.comets.snapshot();
        CometCatalogMapper::to_dto(&status, Utc::now())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clients::mpc_client::parse_comet_elements;
    use crate::dto::{BodyTypeDto, VisibilityDto};
    use crate::facades::observer_facade::tests::facade_at;
    use crate::models::Observer;

    const COMETS: &str = "\
0002P         2027 02 10.2274  0.338598  0.847321  187.2875  334.0199   11.3480  20261030   5.0  4.0  2P/Encke                                                 MPEC 2026-UA3
    CJ95O010  1997 03 29.0271  0.926428  0.994894  130.7350  281.8122   89.7323  20261030  -2.0  4.0  C/1995 O1 (Hale-Bopp)                                    MPC194091
";

    fn facade() -> CelestialBodyFacade {
        CelestialBodyFacade::new(
            Arc::new(Catalogs::load().unwrap()),
            facade_at(Observer::paris()),
            Arc::new(CometCatalog::fixed(parse_comet_elements(COMETS))),
        )
    }

    #[test]
    fn lists_moon_planets_and_bright_comets() {
        let bodies = facade().get_visible_bodies(Some("2026-10-30T20:00:00Z")).unwrap();
        // Lune + 7 planètes + 2P/Encke (Hale-Bopp, à plus de 40 UA, est trop faible).
        assert_eq!(bodies.len(), 9);
        assert_eq!(bodies[0].body_type, BodyTypeDto::Moon);
        assert!(bodies[0].distance_km.is_some());
        let encke = bodies.iter().find(|b| b.body_type == BodyTypeDto::Comet).unwrap();
        assert_eq!(encke.id, "comet:2P/Encke");
        assert!(encke.magnitude <= COMET_LIMIT);

        let saturn = bodies.iter().find(|b| b.id == "saturn").unwrap();
        assert_eq!(saturn.status, VisibilityDto::Visible);
        // Confirmé par astropy.coordinates.get_constellation (mêmes limites de Roman).
        assert_eq!(saturn.constellation.as_ref().unwrap().abbreviation, "Cet");
        let neptune = bodies.iter().find(|b| b.id == "neptune").unwrap();
        assert_eq!(neptune.status, VisibilityDto::TooFaint);
    }

    #[test]
    fn computes_tonight_events() {
        let bodies = facade().get_visible_bodies(Some("2026-10-30T20:00:00Z")).unwrap();
        let saturn = bodies.iter().find(|b| b.id == "saturn").unwrap();
        let night = &saturn.night;
        assert!(night.sunset.as_deref().unwrap().starts_with("2026-10-30T16:3"));
        assert!(night.sunrise.as_deref().unwrap().starts_with("2026-10-31T06:3"));
        assert_eq!(night.altitudes.len(), NIGHT_CURVE_POINTS);
        // Saturne culmine en soirée à 90° − 48.9° + 1.2° ≈ 42.4°.
        let transit_altitude = saturn.transit_altitude_deg.unwrap();
        assert!((transit_altitude - 42.4).abs() < 0.5, "culmination {transit_altitude}");
        assert!(saturn.rise.is_some() && saturn.set.is_some());
        assert!(saturn.max_night_altitude_deg.unwrap() >= night.altitudes.iter().copied().fold(f64::MIN, f64::max) - 1e-6);
    }

    #[test]
    fn mercury_and_venus_have_phases() {
        let bodies = facade().get_visible_bodies(Some("2026-10-30T20:00:00Z")).unwrap();
        for id in ["mercury", "venus", "moon"] {
            assert!(bodies.iter().find(|b| b.id == id).unwrap().illumination.is_some(), "{id}");
        }
        assert!(bodies.iter().find(|b| b.id == "mars").unwrap().illumination.is_none());
        assert!(bodies[0].phase_name.is_some());
    }
}
