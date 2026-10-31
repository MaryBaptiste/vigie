use std::sync::Arc;

use crate::astronomy::coordinates::{spherical_to_vector, vector_to_spherical};
use crate::astronomy::linalg::{Vec3, add};
use crate::astronomy::stars::apparent_place;
use crate::astronomy::time::julian_day;
use crate::astronomy::{Equatorial, Horizontal, Planet};
use crate::catalogs::Catalogs;
use crate::dto::SkyDto;
use crate::error::AppResult;
use crate::facades::ephemeris::{COMET_LIMIT, Situation, Target, refracted};
use crate::facades::{CometCatalog, ObserverFacade, parse_instant};
use crate::mappers::SkyMapper;
use crate::models::{Constellation, SkySnapshot, SkyState, VisibleConstellation, VisibleSegment, VisibleStar};

pub struct SkyFacade {
    catalogs: Arc<Catalogs>,
    observer: ObserverFacade,
    comets: Arc<CometCatalog>,
}

impl SkyFacade {
    pub fn new(catalogs: Arc<Catalogs>, observer: ObserverFacade, comets: Arc<CometCatalog>) -> Self {
        Self { catalogs, observer, comets }
    }

    /// Ciel vu par l'observateur courant, à l'instant demandé ou maintenant.
    pub fn get_sky(&self, datetime_utc: Option<&str>) -> AppResult<SkyDto> {
        let instant = parse_instant(datetime_utc)?;
        let observer = self.observer.current();
        let situation = Situation::new(julian_day(&instant), observer.site());

        let stars = self
            .catalogs
            .stars
            .iter()
            .filter_map(|star| {
                let place = apparent_place(&star.position, &situation.epoch);
                let horizontal = refracted(situation.geometric_horizontal(place));
                (horizontal.altitude_deg >= 0.0).then_some(VisibleStar { star, horizontal })
            })
            .collect();

        let constellations =
            self.catalogs.constellations.iter().filter_map(|c| visible_constellation(&situation, c)).collect();

        let (comets, _) = self.comets.snapshot();
        let targets = [Target::Sun, Target::Moon]
            .into_iter()
            .chain(Planet::ALL.map(Target::Planet))
            .chain(
                comets
                    .iter()
                    .filter(|comet| situation.comet_magnitude(comet) <= COMET_LIMIT)
                    .map(Target::Comet),
            );
        let bodies = targets.map(|target| situation.body(&target)).collect();

        let sun_altitude_deg = situation.sun_altitude_deg();
        let snapshot = SkySnapshot {
            computed_at: instant,
            local_sidereal_deg: situation.local_sidereal_deg,
            observer,
            sky_state: SkyState::from_sun_altitude(sun_altitude_deg),
            sun_altitude_deg,
            stars,
            constellations,
            bodies,
        };
        Ok(SkyMapper::to_dto(&snapshot))
    }
}

/// Segments dont au moins une extrémité est visible (le front découpe ceux qui
/// traversent l'horizon), et libellé au barycentre des sommets visibles.
fn visible_constellation<'a>(situation: &Situation, constellation: &'a Constellation) -> Option<VisibleConstellation<'a>> {
    let to_horizontal = |p: Equatorial| situation.apparent_horizontal_of_j2000(p.to_vector());
    let mut barycentre: Vec3 = [0.0; 3];
    let mut visible_vertices = 0;
    let mut accumulate = |h: &Horizontal| {
        if h.altitude_deg >= 0.0 {
            // Repère local : x vers le nord, y vers l'est (sens de l'azimut).
            barycentre = add(barycentre, spherical_to_vector(h.azimuth_deg, h.altitude_deg));
            visible_vertices += 1;
        }
    };

    let segments: Vec<VisibleSegment> = constellation
        .segments
        .iter()
        .filter_map(|segment| {
            let from = to_horizontal(segment.from);
            let to = to_horizontal(segment.to);
            if from.altitude_deg < 0.0 && to.altitude_deg < 0.0 {
                return None;
            }
            accumulate(&from);
            accumulate(&to);
            Some(VisibleSegment { id: segment.id, from, to })
        })
        .collect();

    if segments.is_empty() {
        return None;
    }
    let label = (visible_vertices > 0).then(|| {
        let (azimuth_deg, altitude_deg) = vector_to_spherical(barycentre);
        Horizontal { azimuth_deg, altitude_deg }
    });
    Some(VisibleConstellation { constellation, segments, label })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dto::{BodyTypeDto, SkyStateDto};
    use crate::facades::observer_facade::tests::facade_at;
    use crate::models::Observer;

    fn facade(observer: Observer) -> SkyFacade {
        SkyFacade::new(
            Arc::new(Catalogs::load().unwrap()),
            facade_at(observer),
            Arc::new(CometCatalog::fixed(vec![])),
        )
    }

    fn sydney() -> Observer {
        Observer { latitude_deg: -33.8688, longitude_deg: 151.2093, altitude_m: 58.0, label: "Sydney".into() }
    }

    #[test]
    fn polaris_stands_due_north_above_paris() {
        let sky = facade(Observer::paris()).get_sky(Some("2026-10-30T20:00:00Z")).unwrap();
        let polaris = sky.stars.iter().find(|s| s.id == 424).expect("Polaire visible");
        assert_eq!(polaris.name.as_deref(), Some("Polaire"));
        assert!((polaris.altitude_deg - 48.9).abs() < 0.5, "hauteur {}", polaris.altitude_deg);
        let azimuth = crate::astronomy::angles::normalize_signed_degrees(polaris.azimuth_deg);
        assert!(azimuth.abs() < 1.5, "azimut {azimuth}");
        assert_eq!(sky.sky_state, SkyStateDto::Night);
        assert!((sky.local_sidereal_time_hours - 22.767).abs() < 0.01);
    }

    #[test]
    fn southern_cross_but_no_polaris_from_sydney() {
        // Avril, 22 h à Sydney : la Croix du Sud est haute dans le ciel.
        let sky = facade(sydney()).get_sky(Some("2026-04-15T12:00:00Z")).unwrap();
        assert!(sky.stars.iter().all(|s| s.id != 424), "Polaire sous l'horizon");
        assert!(sky.stars.iter().any(|s| s.id == 4730), "Acrux visible");
        let crux = sky.constellations.iter().find(|c| c.abbreviation == "Cru").expect("Croix du Sud");
        assert_eq!(crux.french_name, "Croix du Sud");
        assert!(crux.label.unwrap().altitude_deg > 30.0);
    }

    #[test]
    fn keeps_only_segments_with_a_visible_end() {
        let sky = facade(Observer::paris()).get_sky(Some("2026-10-30T20:00:00Z")).unwrap();
        for constellation in &sky.constellations {
            for segment in &constellation.segments {
                assert!(segment.from.altitude_deg >= 0.0 || segment.to.altitude_deg >= 0.0);
            }
        }
        assert!(sky.stars.iter().all(|s| s.altitude_deg >= 0.0));
        // Environ la moitié du catalogue est au-dessus de l'horizon.
        assert!((1800..3300).contains(&sky.stars.len()), "{} étoiles", sky.stars.len());
    }

    #[test]
    fn includes_sun_moon_and_planets() {
        let sky = facade(Observer::paris()).get_sky(Some("2026-10-30T20:00:00Z")).unwrap();
        assert_eq!(sky.bodies.len(), 9);
        assert_eq!(sky.bodies[0].body_type, BodyTypeDto::Sun);
        let moon = sky.bodies.iter().find(|b| b.id == "moon").unwrap();
        assert!(moon.illumination.is_some());
    }
}
