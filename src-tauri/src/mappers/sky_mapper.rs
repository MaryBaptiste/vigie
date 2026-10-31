use crate::astronomy::Horizontal;
use crate::dto::{ConstellationDto, HorizontalDto, SegmentDto, SkyDto, SkyStateDto, StarDto};
use crate::mappers::{CelestialBodyMapper, ObserverMapper, iso, round4};
use crate::models::{SkySnapshot, SkyState, VisibleConstellation, VisibleStar};

pub struct SkyMapper;

impl SkyMapper {
    pub fn to_dto(snapshot: &SkySnapshot) -> SkyDto {
        SkyDto {
            timestamp: iso(&snapshot.computed_at),
            local_sidereal_time_hours: snapshot.local_sidereal_deg / 15.0,
            observer: ObserverMapper::to_dto(&snapshot.observer),
            sky_state: Self::sky_state(snapshot.sky_state),
            sun_altitude_deg: round4(snapshot.sun_altitude_deg),
            stars: snapshot.stars.iter().map(Self::star).collect(),
            constellations: snapshot.constellations.iter().map(Self::constellation).collect(),
            bodies: snapshot.bodies.iter().map(CelestialBodyMapper::to_sky_dto).collect(),
        }
    }

    fn sky_state(state: SkyState) -> SkyStateDto {
        match state {
            SkyState::Day => SkyStateDto::Day,
            SkyState::CivilTwilight => SkyStateDto::CivilTwilight,
            SkyState::NauticalTwilight => SkyStateDto::NauticalTwilight,
            SkyState::AstronomicalTwilight => SkyStateDto::AstronomicalTwilight,
            SkyState::Night => SkyStateDto::Night,
        }
    }

    fn horizontal(position: &Horizontal) -> HorizontalDto {
        HorizontalDto {
            azimuth_deg: round4(position.azimuth_deg),
            altitude_deg: round4(position.altitude_deg),
        }
    }

    fn star(visible: &VisibleStar) -> StarDto {
        StarDto {
            id: visible.star.id,
            name: visible.star.name.clone(),
            azimuth_deg: round4(visible.horizontal.azimuth_deg),
            altitude_deg: round4(visible.horizontal.altitude_deg),
            magnitude: visible.star.magnitude,
            color_index: visible.star.color_index,
        }
    }

    fn constellation(visible: &VisibleConstellation) -> ConstellationDto {
        let c = visible.constellation;
        ConstellationDto {
            abbreviation: c.abbreviation.clone(),
            latin_name: c.latin_name.clone(),
            french_name: c.french_name.clone(),
            segments: visible
                .segments
                .iter()
                .map(|s| SegmentDto { id: s.id, from: Self::horizontal(&s.from), to: Self::horizontal(&s.to) })
                .collect(),
            label: visible.label.as_ref().map(Self::horizontal),
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use super::*;
    use crate::astronomy::stars::CatalogPosition;
    use crate::models::{Constellation, Observer, Star, VisibleSegment};

    #[test]
    fn maps_snapshot() {
        let star = Star {
            id: 7001,
            name: Some("Véga".into()),
            position: CatalogPosition {
                ra_deg: 279.2,
                dec_deg: 38.8,
                pm_ra_arcsec_per_year: 0.0,
                pm_dec_arcsec_per_year: 0.0,
            },
            magnitude: 0.03,
            color_index: Some(0.0),
        };
        let lyra = Constellation {
            abbreviation: "Lyr".into(),
            latin_name: "Lyra".into(),
            french_name: "Lyre".into(),
            segments: vec![],
        };
        let up = Horizontal { azimuth_deg: 281.280_94, altitude_deg: 45.426_06 };
        let down = Horizontal { azimuth_deg: 10.0, altitude_deg: -2.0 };
        let snapshot = SkySnapshot {
            computed_at: Utc.with_ymd_and_hms(2026, 10, 30, 20, 0, 0).unwrap(),
            local_sidereal_deg: 355.5,
            observer: Observer::paris(),
            sky_state: SkyState::Night,
            sun_altitude_deg: -34.1,
            stars: vec![VisibleStar { star: &star, horizontal: up }],
            constellations: vec![VisibleConstellation {
                constellation: &lyra,
                segments: vec![VisibleSegment { id: 4, from: up, to: down }],
                label: Some(up),
            }],
            bodies: vec![],
        };

        let json = serde_json::to_value(SkyMapper::to_dto(&snapshot)).unwrap();
        assert_eq!(json["timestamp"], "2026-10-30T20:00:00Z");
        assert_eq!(json["skyState"], "NUIT");
        assert_eq!(json["localSiderealTimeHours"], 23.7);
        assert_eq!(json["stars"][0]["name"], "Véga");
        assert_eq!(json["stars"][0]["azimuthDeg"], 281.2809);
        assert_eq!(json["constellations"][0]["frenchName"], "Lyre");
        assert_eq!(json["constellations"][0]["segments"][0]["to"]["altitudeDeg"], -2.0);
        assert_eq!(json["observer"]["label"], "Paris");
    }
}
