//! Données embarquées dans l'exécutable (générées par `tools/build_catalogs.py`),
//! chargées une fois au démarrage.

use std::collections::HashMap;

use serde::Deserialize;

use crate::astronomy::Equatorial;
use crate::astronomy::constellation::{BoundaryRow, constellation_of};
use crate::astronomy::stars::CatalogPosition;
use crate::error::{AppError, AppResult};
use crate::models::{Constellation, ConstellationSegment, Star};

const STARS_JSON: &str = include_str!("../../data/stars.json");
const CONSTELLATIONS_JSON: &str = include_str!("../../data/constellations.json");
const BOUNDARIES_JSON: &str = include_str!("../../data/boundaries.json");

/// HR, α J2000 (°), δ J2000 (°), μα·cos δ (″/an), μδ (″/an), V, B−V, nom.
type RawStar = (u32, f64, f64, f64, f64, f64, Option<f64>, Option<String>);
/// AD basse (h), AD haute (h), δ basse (°), abréviation.
type RawBoundary = (f64, f64, f64, String);

#[derive(Deserialize)]
struct RawConstellation {
    abbr: String,
    latin: String,
    french: String,
    /// Polylignes de sommets [α, δ] J2000.
    lines: Vec<Vec<[f64; 2]>>,
}

pub struct Catalogs {
    pub stars: Vec<Star>,
    pub constellations: Vec<Constellation>,
    boundaries: Vec<BoundaryRow>,
}

impl Catalogs {
    pub fn load() -> AppResult<Self> {
        let constellations = parse_constellations(CONSTELLATIONS_JSON)?;
        Ok(Self {
            stars: parse_stars(STARS_JSON)?,
            boundaries: parse_boundaries(BOUNDARIES_JSON, &constellations)?,
            constellations,
        })
    }

    /// Constellation IAU contenant une position rapportée à l'équinoxe `epoch_jde`.
    pub fn constellation_at(&self, position: Equatorial, epoch_jde: f64) -> Option<&Constellation> {
        constellation_of(&self.boundaries, position, epoch_jde).map(|index| &self.constellations[index])
    }
}

fn corrupted(name: &str, error: impl std::fmt::Display) -> AppError {
    AppError::Catalog(format!("{name}: {error}"))
}

fn parse_stars(json: &str) -> AppResult<Vec<Star>> {
    let raw: Vec<RawStar> = serde_json::from_str(json).map_err(|e| corrupted("stars.json", e))?;
    Ok(raw
        .into_iter()
        .map(|(id, ra_deg, dec_deg, pm_ra, pm_dec, magnitude, color_index, name)| Star {
            id,
            name,
            position: CatalogPosition {
                ra_deg,
                dec_deg,
                pm_ra_arcsec_per_year: pm_ra,
                pm_dec_arcsec_per_year: pm_dec,
            },
            magnitude,
            color_index,
        })
        .collect())
}

fn parse_constellations(json: &str) -> AppResult<Vec<Constellation>> {
    let raw: Vec<RawConstellation> = serde_json::from_str(json).map_err(|e| corrupted("constellations.json", e))?;
    let mut next_id = 0;
    Ok(raw
        .into_iter()
        .map(|c| {
            let vertex = |[ra_deg, dec_deg]: [f64; 2]| Equatorial { ra_deg, dec_deg };
            let segments = c
                .lines
                .iter()
                .flat_map(|polyline| polyline.windows(2))
                .map(|pair| {
                    next_id += 1;
                    ConstellationSegment { id: next_id, from: vertex(pair[0]), to: vertex(pair[1]) }
                })
                .collect();
            Constellation {
                abbreviation: c.abbr,
                latin_name: c.latin,
                french_name: c.french,
                segments,
            }
        })
        .collect())
}

fn parse_boundaries(json: &str, constellations: &[Constellation]) -> AppResult<Vec<BoundaryRow>> {
    let index: HashMap<&str, usize> =
        constellations.iter().enumerate().map(|(i, c)| (c.abbreviation.as_str(), i)).collect();
    let raw: Vec<RawBoundary> = serde_json::from_str(json).map_err(|e| corrupted("boundaries.json", e))?;
    raw.into_iter()
        .map(|(ra_low_hours, ra_high_hours, dec_low_deg, abbr)| {
            let constellation = *index
                .get(abbr.as_str())
                .ok_or_else(|| corrupted("boundaries.json", format!("unknown constellation {abbr}")))?;
            Ok(BoundaryRow { ra_low_hours, ra_high_hours, dec_low_deg, constellation })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::astronomy::time::J2000;

    fn catalogs() -> Catalogs {
        Catalogs::load().expect("catalogues embarqués valides")
    }

    #[test]
    fn loads_embedded_catalogs() {
        let c = catalogs();
        assert!((4900..5200).contains(&c.stars.len()));
        assert!(c.stars.iter().all(|s| s.magnitude <= 6.0));
        assert_eq!(c.constellations.len(), 88);
        assert!(c.constellations.iter().all(|c| !c.segments.is_empty()));
        let sirius = c.stars.iter().find(|s| s.id == 2491).unwrap();
        assert_eq!(sirius.name.as_deref(), Some("Sirius"));
        let vega = c.stars.iter().find(|s| s.id == 7001).unwrap();
        assert_eq!(vega.name.as_deref(), Some("Véga"));
        let big_dipper = c.constellations.iter().find(|c| c.abbreviation == "UMa").unwrap();
        assert_eq!(big_dipper.french_name, "Grande Ourse");
    }

    #[test]
    fn segment_ids_are_unique() {
        let c = catalogs();
        let mut ids: Vec<u32> = c.constellations.iter().flat_map(|c| c.segments.iter().map(|s| s.id)).collect();
        let count = ids.len();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), count);
    }

    #[test]
    fn every_bright_star_lies_in_its_constellation() {
        // Contrôle croisé : limites IAU (Roman) contre la constellation des étoiles nommées.
        let c = catalogs();
        let cases = [(424, "UMi"), (7001, "Lyr"), (2491, "CMa"), (4730, "Cru"), (2061, "Ori"), (5340, "Boo")];
        for (hr, expected) in cases {
            let star = c.stars.iter().find(|s| s.id == hr).unwrap();
            let position = Equatorial { ra_deg: star.position.ra_deg, dec_deg: star.position.dec_deg };
            let found = c.constellation_at(position, J2000).map(|c| c.abbreviation.as_str());
            assert_eq!(found, Some(expected), "HR {hr}");
        }
    }

    #[test]
    fn whole_sky_is_covered() {
        let c = catalogs();
        for ra in (0..360).step_by(7) {
            for dec in (-89..=89).step_by(5) {
                let position = Equatorial { ra_deg: f64::from(ra), dec_deg: f64::from(dec) };
                assert!(c.constellation_at(position, J2000).is_some(), "{ra} {dec}");
            }
        }
    }
}
